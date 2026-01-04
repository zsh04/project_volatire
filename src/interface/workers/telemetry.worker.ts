
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
}

interface WorkerMessage {
    type: 'MARKET_TICK' | 'ERROR';
    payload?: PhysicsData;
    error?: string;
}

// Connect directly to Envoy proxy
// Note: This works because the worker is loaded from the same origin
const ENVOY_URL = process.env.NEXT_PUBLIC_ENVOY_URL || 'http://localhost:8080';
const client = new ReflexServiceClient(ENVOY_URL, null, null);

// Stream Control
let stream: any = null;

// Start Ingestion
self.onmessage = (e: MessageEvent) => {
    if (e.data.type === 'START') {
        startStream();
    } else if (e.data.type === 'STOP') {
        stopStream();
    }
};

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
        console.log('✅ [Worker] Stream data received!', res);

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
        };

        // Zero-Latency Pass-Through
        self.postMessage({ type: 'MARKET_TICK', payload: data });
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
