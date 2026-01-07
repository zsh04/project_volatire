import { ReflexServiceClient } from './generated/ReflexServiceClientPb';
import { Empty, PhysicsResponse, OODAResponse, VetoRequest, LegislativeUpdate } from './generated/reflex_pb';
import { useMarketStore } from '../stores/market-store';
import { useAgentStore } from '../stores/agent-store';
import { useSystemStore } from '../stores/system-store';

// ==============================================================================
// Client Configuration
// ==============================================================================

const ENVOY_URL = process.env.NEXT_PUBLIC_ENVOY_URL || 'http://localhost:8080';

// Singleton client instance
let clientInstance: ReflexServiceClient | null = null;

export function getReflexClient(): ReflexServiceClient {
    if (!clientInstance) {
        clientInstance = new ReflexServiceClient(ENVOY_URL, null, null);
    }
    return clientInstance;
}

// ==============================================================================
// Telemetry Streaming
// ==============================================================================

export class TelemetryStream {
    private client: ReflexServiceClient;
    private physicsInterval: NodeJS.Timeout | null = null;
    private oodaInterval: NodeJS.Timeout | null = null;

    constructor() {
        this.client = getReflexClient();
    }

    /**
     * Start streaming telemetry from Reflex
     * Polls GetPhysics and GetOODA at high frequency
     */
    start() {
        console.log('🔌 Starting telemetry stream from Reflex...');

        // Physics updates every 150ms (Tick-to-ACK budget)
        this.physicsInterval = setInterval(() => {
            const request = new Empty();

            this.client.getPhysics(request, {}, (err, response: PhysicsResponse) => {
                if (err) {
                    console.error('Physics stream error:', err);
                    return;
                }

                // Update market store
                useMarketStore.getState().updatePhysics({
                    price: response.getPrice(),
                    velocity: response.getVelocity(),
                    acceleration: response.getAcceleration(),
                    jerk: response.getJerk(),
                    entropy: response.getEntropy(),
                    efficiencyIndex: response.getEfficiencyIndex(),
                    timestamp: response.getTimestamp(),
                });

                // D-90: Update System Sanity Score
                useSystemStore.getState().setSystemSanityScore(response.getSystemSanityScore());

                // Compute Riemann state from efficiency index
                const efficiency = response.getEfficiencyIndex();
                const riemannState = efficiency > 0.7 ? 'momentum' :
                    efficiency < 0.3 ? 'meanReversion' :
                        'transitioning';
                useMarketStore.getState().updateRiemann(riemannState, efficiency);
            });
        }, 150);

        // OODA updates every 1s
        this.oodaInterval = setInterval(() => {
            const request = new Empty();

            this.client.getOODA(request, {}, (err, response: OODAResponse) => {
                if (err) {
                    console.error('OODA stream error:', err);
                    return;
                }

                // Update agent store
                const decision = response.getDecision();
                const sentimentScore = response.getSentimentScore();
                const nearestRegime = response.getNearestRegime();

                // Parse weights map for persona confidence
                const weightsMap = response.getWeightsMap();
                const simonsWeight = weightsMap.get('Simons') || 0;
                const hypatiaWeight = weightsMap.get('Hypatia') || 0;

                // Update personas
                useAgentStore.getState().updatePersona('Simons', {
                    status: 'active',
                    confidence: simonsWeight,
                    lastReason: `Regime: ${nearestRegime || 'Unknown'}`,
                });

                useAgentStore.getState().updatePersona('Hypatia', {
                    status: 'active',
                    confidence: hypatiaWeight,
                    lastReason: `Sentiment: ${sentimentScore?.toFixed(2) || 'N/A'}`,
                });

                // Calculate consensus (alignment of weights)
                const consensus = 1 - Math.abs(simonsWeight - hypatiaWeight);
                useAgentStore.getState().updateConsensus(consensus);

                // Set decision
                if (decision) {
                    useAgentStore.getState().setDecision({
                        action: decision as 'BUY' | 'SELL' | 'HOLD',
                        reason: `Regime: ${nearestRegime}, Sentiment: ${sentimentScore?.toFixed(2)}`,
                        confidence: Math.max(simonsWeight, hypatiaWeight),
                        timestamp: Date.now(),
                    });

                    // Add to reasoning stream
                    useAgentStore.getState().addReasoning({
                        persona: simonsWeight > hypatiaWeight ? 'Simons' : 'Hypatia',
                        message: `Decision: ${decision} (Confidence: ${(Math.max(simonsWeight, hypatiaWeight) * 100).toFixed(0)}%)`,
                        timestamp: Date.now(),
                        type: decision === 'HOLD' ? 'info' : 'warning',
                    });
                }
            });
        }, 1000);
    }

    /**
     * Stop streaming
     */
    stop() {
        console.log('⏸️ Stopping telemetry stream...');

        if (this.physicsInterval) {
            clearInterval(this.physicsInterval);
            this.physicsInterval = null;
        }

        if (this.oodaInterval) {
            clearInterval(this.oodaInterval);
            this.oodaInterval = null;
        }
    }
}

// ==============================================================================
// Manual Control RPCs
// ==============================================================================

export async function triggerVeto(reason: string, operator: string): Promise<boolean> {
    const client = getReflexClient();

    return new Promise((resolve, reject) => {
        const request = new VetoRequest();
        request.setReason(reason);
        request.setOperator(operator);

        client.triggerVeto(request, {}, (err, response) => {
            if (err) {
                console.error('Veto trigger failed:', err);
                reject(err);
                return;
            }

            // Update agent store
            useAgentStore.getState().activateVeto(reason);
            resolve(true);
        });
    });
}

export async function updateLegislation(
    bias: string,
    aggression: number,
    makerOnly: boolean,
    hibernation: boolean,
    snapToBreakeven: boolean
): Promise<boolean> {
    const client = getReflexClient();

    return new Promise((resolve, reject) => {
        const request = new LegislativeUpdate();
        request.setBias(bias);
        request.setAggression(aggression);
        request.setMakerOnly(makerOnly);
        request.setHibernation(hibernation);
        request.setSnapToBreakeven(snapToBreakeven);

        client.updateLegislation(request, {}, (err, response) => {
            if (err) {
                console.error('Legislation update failed:', err);
                reject(err);
                return;
            }
            resolve(true); // Ack
        });
    });
}
