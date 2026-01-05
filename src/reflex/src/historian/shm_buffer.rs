use std::sync::atomic::Ordering;
use std::fs::OpenOptions;
use std::path::Path;
use memmap2::MmapMut;
use bytemuck::{Pod, Zeroable};
use crate::historian::events::LogEvent;

// Constants
const BUFFER_SIZE: usize = 1024 * 16; // 16k events capacity
pub const SLOT_SIZE: usize = 256; // 256 bytes per event (generous)
const SHM_PATH: &str = "/dev/shm/reflex_log_ring";

// Layout of the Shared Memory Header
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RingHeader {
    pub head: u64, // Monotonically increasing write index
    pub tail: u64, // Monotonically increasing read index (updated by archiver)
    pub dropped: u64, // Counter for dropped events if full
    pub _padding: [u64; 5], // align to cache line (64 bytes)
}

pub struct ShmRingBuffer {
    mmap: MmapMut,
}

impl ShmRingBuffer {
    pub fn new() -> std::io::Result<Self> {
        let path = Path::new(SHM_PATH);
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let total_size = std::mem::size_of::<RingHeader>() + (BUFFER_SIZE * SLOT_SIZE);
        
        file.set_len(total_size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Initialize header if fresh (check if head is 0 and tail is 0, practically)
        // Or we just rely on OS zeroing new files.
        // For robustness in Restart, we should probably read what's there.
        // But for D-82 Init, let's assume valid state or 0.

        Ok(Self { mmap })
    }

    #[inline(always)]
    pub fn write(&mut self, event: &LogEvent) {
        // 1. Get Header (unsafe pointer cast)
        let header_ptr = self.mmap.as_mut_ptr() as *mut RingHeader;
        let header = unsafe { &mut *header_ptr };

        // 2. Check Capacity
        // In a true lock-free SPMC, we load head/tail roughly.
        // Since we are the ONLY producer, we own 'head'.
        // We read 'tail' (volatile load implicitly via reference or strict read).
        // Note: Generic Pod struct fields aren't Atomic, so we use volatile read/write for shared simple types 
        // or we should check if we can cast to atomic.
        // For simplicity in this step, let's just ready directly (x86 TSO usually fine, but volatile is safer).
        
        let head = unsafe { std::ptr::read_volatile(&header.head) };
        let tail = unsafe { std::ptr::read_volatile(&header.tail) };

        if head - tail >= BUFFER_SIZE as u64 {
            // Buffer Full
            unsafe { 
                let d = std::ptr::read_volatile(&header.dropped);
                std::ptr::write_volatile(&mut header.dropped, d + 1);
            }
            return;
        }

        // 3. Serialize to Slot
        // 3. Serialize to Slot (Zero-Copy MEMCPY)
        let slot_idx = (head as usize) % BUFFER_SIZE;
        let offset = std::mem::size_of::<RingHeader>() + (slot_idx * SLOT_SIZE);
        
        // We know slot is aligned to 64 bytes (Header is 64, SLOT_SIZE is 256)
        // So we can cast the pointer to *mut LogEvent
        let dst_ptr = unsafe { self.mmap.as_mut_ptr().add(offset) as *mut LogEvent };
        
        unsafe {
            std::ptr::write(dst_ptr, *event);
        }

        // Write Length? No, fixed size reading based on variant.
        // Or if we want to be safe, we rely on LogEvent being Copy.
        
        // 4. Commit Head

        // 4. Commit Head
        // Write barrier potentially needed, then update head.
        std::sync::atomic::fence(Ordering::Release);
        unsafe { std::ptr::write_volatile(&mut header.head, head + 1) };
    }

    pub fn read_batch(&mut self, max_events: usize) -> Vec<LogEvent> {
        let header_ptr = self.mmap.as_mut_ptr() as *mut RingHeader;
        let header = unsafe { &mut *header_ptr };
        
        let head = unsafe { std::ptr::read_volatile(&header.head) };
        let tail = unsafe { std::ptr::read_volatile(&header.tail) };
        
        if head <= tail {
            return Vec::new();
        }

        let count = std::cmp::min((head - tail) as usize, max_events);
        let mut events = Vec::with_capacity(count);

        for i in 0..count {
            let current_idx = tail + i as u64;
            let slot_idx = (current_idx as usize) % BUFFER_SIZE;
            let offset = std::mem::size_of::<RingHeader>() + (slot_idx * SLOT_SIZE);
            
            let src_ptr = unsafe { self.mmap.as_ptr().add(offset) as *const LogEvent };
            let event = unsafe { std::ptr::read(src_ptr) };
            events.push(event);
        }

        // Commit tail
        unsafe { std::ptr::write_volatile(&mut header.tail, tail + count as u64) };
        
        events
    }
}
