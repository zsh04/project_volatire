use std::mem;

/// SBE (Simple Binary Encoding) Header
/// 4 Bytes: BlockLength (2) + TemplateID (2)
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SbeHeader {
    pub block_length: u16,
    pub template_id: u16,
}

/// payload for a New Order Single (Simplified)
/// 24 Bytes: Header (4) + Price (8) + Qty (8) + Side (4)
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct WirePacket {
    pub header: SbeHeader,
    pub price: f64,
    pub qty: f64,
    pub side: u32, // 1 = Buy, 2 = Sell
}

pub struct BinaryPacker {
    pub buy_buffer: Vec<u8>,
    pub sell_buffer: Vec<u8>,
}

impl BinaryPacker {
    pub fn new() -> Self {
        let mut packer = Self {
            buy_buffer: vec![0u8; mem::size_of::<WirePacket>()],
            sell_buffer: vec![0u8; mem::size_of::<WirePacket>()],
        };
        packer.prepare_templates();
        packer
    }

    fn prepare_templates(&mut self) {
        // Pre-bake BUY Packet
        unsafe {
            let ptr = self.buy_buffer.as_mut_ptr() as *mut WirePacket;
            // Use addr_of_mut! to avoid creating unaligned references
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).header.block_length), 20);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).header.template_id), 99);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).side), 1);
        }

        // Pre-bake SELL Packet
        unsafe {
            let ptr = self.sell_buffer.as_mut_ptr() as *mut WirePacket;
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).header.block_length), 20);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).header.template_id), 99);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).side), 2);
        }
    }

    /// ZERO-COPY UPDATE: Writes Price/Qty directly to the pre-allocated buffer
    /// Uses raw pointer arithmetic to avoid serialization overhead.
    /// Returns the slice ready for "send()"
    #[inline(always)]
    pub fn pack_buy(&mut self, price: f64, qty: f64) -> &[u8] {
        unsafe {
            let ptr = self.buy_buffer.as_mut_ptr() as *mut WirePacket;
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).price), price);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).qty), qty);
        }
        &self.buy_buffer
    }

    #[inline(always)]
    pub fn pack_sell(&mut self, price: f64, qty: f64) -> &[u8] {
        unsafe {
            let ptr = self.sell_buffer.as_mut_ptr() as *mut WirePacket;
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).price), price);
            std::ptr::write_unaligned(std::ptr::addr_of_mut!((*ptr).qty), qty);
        }
        &self.sell_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_layout() {
        assert_eq!(mem::size_of::<SbeHeader>(), 4);
        assert_eq!(mem::size_of::<WirePacket>(), 24); // 4 + 8 + 8 + 4
    }

    #[test]
    fn test_zero_copy_update() {
        let mut packer = BinaryPacker::new();
        
        let price = 50000.50;
        let qty = 1.5;
        
        // Pack Buy
        let buffer = packer.pack_buy(price, qty);
        
        // Verify Size
        assert_eq!(buffer.len(), 24);
        
        // unsafe re-cast to verify content
        let packet = unsafe { &*(buffer.as_ptr() as *const WirePacket) };
        
        let bl = packet.header.block_length;
        let tid = packet.header.template_id;
        let s = packet.side;
        let p = packet.price;
        let q = packet.qty;

        assert_eq!(bl, 20);
        assert_eq!(tid, 99);
        assert_eq!(s, 1);
        assert_eq!(p, price);
        assert_eq!(q, qty);
    }
}
