
import { Empty } from '../lib/grpc/generated/reflex_pb';
import { ReflexServiceClient } from '../lib/grpc/generated/ReflexServiceClientPb';

// Types (Mirroring the proto response structure for transfer)
export interface PhysicsData {
    price: number;
    velocity: number;
    acceleration: number;
    jerk: number;
    entropy: number;
    efficiencyIndex: number;
    timestamp: number;
    // Finance
    unrealizedPnl: number;
    equity: number;
    balance: number;
    // Model
    gemmaTokensPerSec: number;
    gemmaLatencyMs: number;
    // Governance
    staircaseTier: number;
    staircaseProgress: number;
    auditDrift: number;

    // Directive-79: Global Sequence ID
    sequenceId: number;
}

interface WorkerMessage {
    type: 'MARKET_TICK' | 'ERROR';
    payload?: PhysicsData;
    error?: string;
}

// --- Priority Queue (Min-Heap) ---
class PriorityQueue<T> {
    private heap: T[] = [];
    private comparator: (a: T, b: T) => number;

    constructor(comparator: (a: T, b: T) => number) {
        this.comparator = comparator;
    }

    push(item: T) {
        this.heap.push(item);
        this.bubbleUp();
    }

    pop(): T | undefined {
        const top = this.heap[0];
        const bottom = this.heap.pop();
        if (this.heap.length > 0 && bottom !== undefined) {
            this.heap[0] = bottom;
            this.bubbleDown();
        }
        return top;
    }

    peek(): T | undefined {
        return this.heap[0];
    }

    size(): number {
        return this.heap.length;
    }

    private bubbleUp() {
        let node = this.heap.length - 1;
        while (node > 0) {
            const parent = Math.floor((node - 1) / 2);
            if (this.comparator(this.heap[node], this.heap[parent]) < 0) {
                [this.heap[node], this.heap[parent]] = [this.heap[parent], this.heap[node]];
                node = parent;
            } else {
                break;
            }
        }
    }

    private bubbleDown() {
        let node = 0;
        while (node * 2 + 1 < this.heap.length) {
            let left = node * 2 + 1;
            let right = node * 2 + 2;
            let minChild = left;

            if (right < this.heap.length && this.comparator(this.heap[right], this.heap[left]) < 0) {
                minChild = right;
            }

            if (this.comparator(this.heap[minChild], this.heap[node]) < 0) {
                [this.heap[node], this.heap[minChild]] = [this.heap[minChild], this.heap[node]];
                node = minChild;
            } else {
                break;
            }
        }
    }
}

// Connect directly to Envoy proxy
// Note: This works because the worker is loaded from the same origin
const ENVOY_URL = process.env.NEXT_PUBLIC_ENVOY_URL || 'http://localhost:8080';
const client = new ReflexServiceClient(ENVOY_URL, null, null);

// Stream Control
let stream: any = null;

// Forensic Ring Buffer (D-78)
const BUFFER_CAPACITY = 3600; // 60s @ 60Hz
const ringBuffer: PhysicsData[] = []; // Circular Buffer
let writeIndex = 0;
let mode: 'LIVE' | 'SCRUB' = 'LIVE';

// Directive-79: Jitter Buffer State
const JITTER_BUFFER_MS = 50;
const buffer = new PriorityQueue<PhysicsData>((a, b) => a.sequenceId - b.sequenceId);
let expectedSeqId = -1; // -1 means uninitialized
let lastEmitTime = 0;

// Re-Assembler Loop
setInterval(() => {
    processJitterBuffer();
}, 10); // Check every 10ms

function processJitterBuffer() {
    if (mode !== 'LIVE' || buffer.size() === 0) return;

    const now = Date.now();
    let packet = buffer.peek();

    while (packet) {
        // Init logic
        if (expectedSeqId === -1) {
            expectedSeqId = packet.sequenceId;
        }

        // Case 1: In Order
        if (packet.sequenceId === expectedSeqId) {
            buffer.pop(); // Remove
            emit(packet);
            expectedSeqId++;
            packet = buffer.peek();
            continue;
        }

        // Case 2: Duplicate / Late (Already processed)
        if (packet.sequenceId < expectedSeqId) {
            // Drop it
            buffer.pop();
            packet = buffer.peek();
            continue;
        }

        // Case 3: Gap Detected (Future packet)
        // Check if we waited long enough (Starvation / Packet Loss)
        // Note: Using packet timestamp vs system time. 
        // We assume packet.timestamp is close to 'now'.
        // If packet is sitting in buffer for > JITTER_MS? No, we don't track insert time.
        // We track HEAD packet age?
        // Simple heuristic: If buffer is getting too big, or if head packet is "old" enough relative to last emit?
        // Better: Compare packet timestamp to estimated 'now'.

        // Fallback: If gap is huge (> 60Hz * 0.1s = 6 frames), just jump?
        // Or if the head packet is older than (Date.now() - JITTER_MS), force emit.
        // But internal timestamp (Sim time) might drift from wall clock.

        // Force Jump if buffer > 10 items (approx 160ms latency at 60Hz)
        if (buffer.size() > 10) {
            console.warn(`[Jitter] Gap Skipped! Jumping from ${expectedSeqId} to ${packet.sequenceId}, Buffer Size: ${buffer.size()}`);
            expectedSeqId = packet.sequenceId;
            // Next loop iteration will emit it
            continue;
        }

        // Else: Wait for hole to fill (Exit loop)
        break;
    }
}

function emit(data: PhysicsData) {
    // Zero-Latency Pass-Through (Now Jitter Buffered)

    // D-78: Always write to Ring Buffer
    if (ringBuffer.length < BUFFER_CAPACITY) {
        ringBuffer.push(data);
    } else {
        ringBuffer[writeIndex] = data; // Overwrite oldest
        writeIndex = (writeIndex + 1) % BUFFER_CAPACITY;
    }

    // Only emit if in LIVE mode
    if (mode === 'LIVE') {
        self.postMessage({ type: 'MARKET_TICK', payload: data });
    }
}


// Start Ingestion
self.onmessage = (e: MessageEvent) => {
    switch (e.data.type) {
        case 'START':
            startStream();
            break;
        case 'STOP':
            stopStream();
            break;
        case 'SET_MODE':
            mode = e.data.payload.mode;
            console.log(`[Worker] Switched to ${mode} mode`);
            break;
        case 'SCRUB_SEEK':
            handleScrubSeek(e.data.payload.timestamp);
            break;
    }
};

function handleScrubSeek(targetTs: number) {
    if (ringBuffer.length === 0) return;

    // O(N) Search - fast enough for 3600 items
    // Since buffer wraps, simple iteration finds the closests match regardless of order
    let closestFrame = ringBuffer[0];
    let minDiff = Math.abs(ringBuffer[0].timestamp - targetTs);

    for (let i = 1; i < ringBuffer.length; i++) {
        const diff = Math.abs(ringBuffer[i].timestamp - targetTs);
        if (diff < minDiff) {
            minDiff = diff;
            closestFrame = ringBuffer[i];
        }
    }

    self.postMessage({ type: 'MARKET_TICK', payload: closestFrame });
}

function startStream() {
    console.log('[Worker] Connecting to Live Telemetry Stream via gRPC-web...');
    console.log('[Worker] Target URL:', ENVOY_URL);
    console.log('[Worker] Client:', client);

    const metadata = {}; // Removed x-data-origin to avoid CORS preflight cache
    const req = new Empty();

    // Directive-72: Open Persistent Stream
    try {
        stream = client.getStream(req, metadata);
        console.log('[Worker] Stream object created:', stream);
    } catch (error) {
        console.error('[Worker] Failed to create stream:', error);
        return;
    }

    stream.on('data', (res: any) => {
        // Confirmation Log
        // console.log('✅ [Worker] Stream data received!', res);

        // Extract raw data
        const data: PhysicsData = {
            price: res.getPrice(),
            velocity: res.getVelocity(),
            acceleration: res.getAcceleration(),
            jerk: res.getJerk(),
            entropy: res.getEntropy(),
            efficiencyIndex: res.getEfficiencyIndex(),
            timestamp: res.getTimestamp(),
            // Finance
            unrealizedPnl: res.getUnrealizedPnl(),
            equity: res.getEquity(),
            balance: res.getBalance(),
            // Model
            gemmaTokensPerSec: res.getGemmaTokensPerSec(),
            gemmaLatencyMs: res.getGemmaLatencyMs(),
            // Governance
            staircaseTier: res.getStaircaseTier(),
            staircaseProgress: res.getStaircaseProgress(),
            auditDrift: res.getAuditDrift(),
            // D-79
            sequenceId: res.getSequenceId(),
        };

        // Push to Jitter Buffer instead of emitting directly
        buffer.push(data);
    });

    stream.on('status', (status: any) => {
        if (status.code !== 0) {
            console.warn('[Worker] Stream Status:', status);
        }
    });

    stream.on('error', (err: any) => {
        console.error('[Worker] Stream Error:', err);
        self.postMessage({ type: 'ERROR', error: err.message });
        // Retry logic could go here
    });

    stream.on('end', () => {
        console.log('[Worker] Stream Ended');
    });
}

function stopStream() {
    if (stream) {
        stream.cancel();
        stream = null;
    }
    console.log('[Worker] Stream Stopped');
}

// D-84: Synthetic Tick Injection for Stress Testing
let syntheticTickInterval: any = null;
let syntheticSeqId = 1000000; // Start at high value to avoid collision

function startSyntheticTicks(rate: number) {
    // rate is ticks per second
    const intervalMs = 1000 / rate;

    console.log(`[Worker] Starting synthetic tick injection at ${rate} ticks/sec`);

    syntheticTickInterval = setInterval(() => {
        const syntheticData: PhysicsData = {
            price: 50000 + Math.random() * 100,
            velocity: Math.random() * 2 - 1,
            acceleration: Math.random() * 0.5 - 0.25,
            jerk: Math.random() * 0.1 - 0.05,
            entropy: Math.random() * 0.5,
            efficiencyIndex: 0.95 + Math.random() * 0.05,
            timestamp: Date.now(),
            unrealizedPnl: 0,
            equity: 10000,
            balance: 10000,
            gemmaTokensPerSec: 100 + Math.random() * 20,
            gemmaLatencyMs: 50 + Math.random() * 10,
            staircaseTier: 0,
            staircaseProgress: 0,
            auditDrift: 0,
            sequenceId: syntheticSeqId++,
        };

        buffer.push(syntheticData);
    }, intervalMs);
}

function stopSyntheticTicks() {
    if (syntheticTickInterval) {
        clearInterval(syntheticTickInterval);
        syntheticTickInterval = null;
        console.log('[Worker] Stopped synthetic tick injection');
    }
}

// Message Handler
self.addEventListener('message', (event: MessageEvent) => {
    const { type, payload } = event.data;

    switch (type) {
        case 'START_STREAM':
            startStream();
            break;
        case 'STOP_STREAM':
            stopStream();
            break;
        case 'START_SCRUB':
            mode = 'SCRUB';
            break;
        case 'STOP_SCRUB':
            mode = 'LIVE';
            break;
        case 'SEEK_SCRUB':
            seekForensicScrub(payload.timestamp);
            break;
        // D-84: Stress Test Controls
        case 'START_SYNTHETIC_TICKS':
            startSyntheticTicks(payload.rate || 5000);
            break;
        case 'STOP_SYNTHETIC_TICKS':
            stopSyntheticTicks();
            break;
        default:
            console.warn('[Worker] Unknown message type:', type);
    }
});

// Auto-start stream on worker init
startStream();
