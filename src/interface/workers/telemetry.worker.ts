
import { Empty, TickHistoryRequest } from '../lib/grpc/generated/reflex_pb';
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
    realizedPnl: number;
    equity: number;
    balance: number;
    btcPosition: number;
    // D-105
    positions: Array<{
        symbol: string;
        netSize: number;
        avgEntryPrice: number;
        unrealizedPnl: number;
        entryTimestamp: number;
        currentPrice: number;
    }>;
    orders: Array<{
        orderId: string;
        symbol: string;
        side: string;
        quantity: number;
        limitPrice: number;
        status: string;
        timestamp: number;
    }>;
    // Model
    gemmaTokensPerSec: number;
    gemmaLatencyMs: number;
    // Governance
    staircaseTier: number;
    staircaseProgress: number;
    auditDrift: number;
    ignitionStatus: string;

    // Directive-79: Global Sequence ID
    sequenceId: number;

    // Directive-103: Reasoning Stream
    reasoningTrace: Array<{
        id: string;
        author: string;
        content: string;
        timestamp: number;
        status: string;
        confidence: number;
    }>;
}

interface WorkerMessage {
    type: 'MARKET_TICK' | 'ERROR';
    payload?: PhysicsData;
    error?: string;
}

// Helper: Parse Physics Response
function parsePhysicsResponse(res: any): PhysicsData {
    return {
        price: res.getPrice(),
        velocity: res.getVelocity(),
        acceleration: res.getAcceleration(),
        jerk: res.getJerk(),
        entropy: res.getEntropy(),
        efficiencyIndex: res.getEfficiencyIndex(),
        timestamp: res.getTimestamp(),
        // Finance
        unrealizedPnl: res.getUnrealizedPnl(),
        realizedPnl: (res.getRealizedPnl && typeof res.getRealizedPnl === 'function') ? res.getRealizedPnl() : 0,
        equity: res.getEquity(),
        balance: res.getBalance(),
        btcPosition: (res.getBtcPosition && typeof res.getBtcPosition === 'function') ? res.getBtcPosition() : 0,

        // Directive-105: Fiscal Deck
        positions: (res.getPositionsList && typeof res.getPositionsList === 'function') ? res.getPositionsList().map((p: any) => ({
            symbol: p.getSymbol(),
            netSize: p.getNetSize(),
            avgEntryPrice: p.getAvgEntryPrice(),
            unrealizedPnl: p.getUnrealizedPnl(),
            entryTimestamp: p.getEntryTimestamp(),
            currentPrice: p.getCurrentPrice()
        })) : [],
        orders: (res.getOrdersList && typeof res.getOrdersList === 'function') ? res.getOrdersList().map((o: any) => ({
            orderId: o.getOrderId(),
            symbol: o.getSymbol(),
            side: o.getSide(),
            quantity: o.getQuantity(),
            limitPrice: o.getLimitPrice(),
            status: o.getStatus(),
            timestamp: o.getTimestamp()
        })) : [],

        // Model
        gemmaTokensPerSec: res.getGemmaTokensPerSec(),
        gemmaLatencyMs: res.getGemmaLatencyMs(),
        // Governance
        staircaseTier: res.getStaircaseTier(),
        staircaseProgress: res.getStaircaseProgress(),
        auditDrift: res.getAuditDrift(),
        ignitionStatus: (res.getIgnitionStatus && typeof res.getIgnitionStatus === 'function') ? res.getIgnitionStatus() : 'HIBERNATION',
        // D-79
        sequenceId: res.getSequenceId(),
        // D-103
        reasoningTrace: res.getReasoningTraceList().map((t: any) => ({
            id: t.getId(),
            author: t.getAuthor(),
            content: t.getContent(),
            timestamp: t.getTimestamp(),
            status: t.getStatus(),
            confidence: t.getConfidence()
        })),
    };
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
let replayStream: any = null;

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
        case 'START_REPLAY':
            startReplay(e.data.payload.symbol, e.data.payload.startTime, e.data.payload.endTime);
            break;
        case 'STOP_REPLAY':
            stopReplay();
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

function startReplay(symbol: string, startTime: number, endTime: number) {
    console.log(`[Worker] Starting Replay for ${symbol} [${startTime} -> ${endTime}]`);
    stopStream(); // Ensure live is off
    if (replayStream) {
        replayStream.cancel();
    }

    // Clear Buffers
    ringBuffer.length = 0;
    writeIndex = 0;
    mode = 'SCRUB'; // Replay implies scrubbing capability

    const req = new TickHistoryRequest();
    req.setSymbol(symbol);
    req.setStartTime(startTime);
    req.setEndTime(endTime);

    try {
        replayStream = client.getTickHistory(req, {});
    } catch (err) {
        console.error('[Worker] Failed to start replay stream:', err);
        return;
    }

    replayStream.on('data', (res: any) => {
        const data = parsePhysicsResponse(res);
        // In replay, we fill the Ring Buffer directly
        if (ringBuffer.length < BUFFER_CAPACITY) {
            ringBuffer.push(data);
        } else {
            // Buffer full? If historical data > 3600 points, we wrap or drop?
            // For now, wrap.
            ringBuffer[writeIndex] = data;
            writeIndex = (writeIndex + 1) % BUFFER_CAPACITY;
        }

        // Auto-emit first frame?
        // Let's emit the first frame so UI shows something
        if (ringBuffer.length === 1) {
            self.postMessage({ type: 'MARKET_TICK', payload: data });
        }
    });

    replayStream.on('end', () => {
        console.log('[Worker] Replay Download Complete. Frames:', ringBuffer.length);
    });

    replayStream.on('error', (err: any) => {
        console.error('[Worker] Replay Stream Error:', err);
    });
}

function stopReplay() {
    if (replayStream) {
        replayStream.cancel();
        replayStream = null;
    }
    mode = 'LIVE';
    startStream(); // Resume live
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
        const data = parsePhysicsResponse(res);
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
            realizedPnl: 0,
            equity: 10000,
            balance: 10000,
            btcPosition: 0.5,
            gemmaTokensPerSec: 100 + Math.random() * 20,
            gemmaLatencyMs: 50 + Math.random() * 10,
            staircaseTier: 0,
            staircaseProgress: 0,
            auditDrift: 0,
            ignitionStatus: 'IGNITED',
            sequenceId: syntheticSeqId++,
            // D-105
            positions: [],
            orders: [],
            reasoningTrace: [],
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

// Auto-start stream on worker init
startStream();
