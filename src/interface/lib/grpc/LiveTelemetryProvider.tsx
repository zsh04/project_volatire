'use client';

import { useEffect, useRef } from 'react';
import { useMarketStore } from '../stores/market-store';
import { useAgentStore } from '../stores/agent-store';
import { useSystemStore } from '../stores/system-store';

export function LiveTelemetryProvider({ children }: { children: React.ReactNode }) {
    const workerRef = useRef<Worker | null>(null);
    const lastPayloadRef = useRef<any>(null);
    const throttleRef = useRef<number | null>(null);

    // 1. Worker Setup for High-Frequency Market Data
    useEffect(() => {
        workerRef.current = new Worker(new URL('../../workers/telemetry.worker.ts', import.meta.url));

        workerRef.current.onmessage = (event) => {
            const { type, payload } = event.data;
            if (type === 'MARKET_TICK') {
                // Directive-72: ALL DATA FROM LIVE gRPC STREAM
                // Throttle updates to ~10Hz (100ms) to prevent UI thread blocking
                // This accumulation strategy prevents dropped packets but updates UI less frequently

                // Store latest payload in ref
                lastPayloadRef.current = payload;
                if (!throttleRef.current) {
                    throttleRef.current = requestAnimationFrame(() => {
                        const p = lastPayloadRef.current;
                        if (!p) return;

                        const now = Date.now();
                        const exchangeTs = p.timestamp || now;
                        const delta = Math.max(0, now - exchangeTs);

                        // Batch Store Updates
                        useMarketStore.getState().updatePhysics({
                            price: Number(p.price) || 0,
                            velocity: Number(p.velocity) || 0,
                            entropy: Number(p.entropy) || 0
                        });

                        useSystemStore.getState().updateFinance({
                            unrealizedPnl: Number(p.unrealizedPnl) || 0,
                            realizedPnl: Number(p.realizedPnl) || 0,
                            equity: Number(p.equity) || 0,
                            balance: Number(p.balance) || 0,
                            btcPosition: Number(p.btcPosition) || 0
                        });

                        // D-103: Live Reasoning Stream
                        if (p.reasoningTrace && Array.isArray(p.reasoningTrace)) {
                            p.reasoningTrace.forEach((event: any) => {
                                useSystemStore.getState().addReasoningEvent(event);

                                // D-104: Bridge to Agent Store for ConsensusMeter
                                if (event.author === 'SOROS' || event.author === 'SIMONS') {
                                    useAgentStore.getState().updatePersona('Simons', {
                                        status: 'active',
                                        lastReason: event.content,
                                        confidence: event.confidence || 0.5
                                    });
                                } else if (event.author === 'TALEB' || event.author === 'HYPATIA') {
                                    useAgentStore.getState().updatePersona('Hypatia', {
                                        status: 'active',
                                        lastReason: event.content,
                                        confidence: event.confidence || 0.5
                                    });
                                }
                            });
                        }

                        // D-83: Ignition Status
                        if (p.ignitionStatus) {
                            useSystemStore.getState().setIgnitionStatus(p.ignitionStatus as any);
                        }

                        const gemmaStatus = (p.gemmaLatencyMs || 0) > 0 ? 'online' : 'offline';
                        useSystemStore.getState().updateNode('gemma', {
                            status: gemmaStatus,
                            metricValue: Number(p.gemmaTokensPerSec) || 0,
                            latency: Number(p.gemmaLatencyMs) || 0
                        });

                        const tierMap = ['Q0', 'Q1', 'Q2', 'Q3', 'Q4', 'MAX'];
                        const safeTierIndex = Math.max(0, Math.min(Number(p.staircaseTier) || 0, 5));
                        useSystemStore.getState().updateStaircase({
                            currentTier: tierMap[safeTierIndex] as any || 'Q0',
                            nextTierProgress: Number(p.staircaseProgress) || 0
                        });

                        useSystemStore.getState().updateAudit({
                            driftScore: Number(p.auditDrift) || 0,
                        });

                        useSystemStore.getState().updateVenue({
                            feedLatency: delta,
                            isDesynced: delta > 500,
                            lastUpdate: now
                        });

                        throttleRef.current = null;
                    });
                }
            }
        };

        workerRef.current.postMessage({ type: 'START', payload: { interval: 16 } });

        return () => {
            console.log('🛑 Terminating Telemetry Worker...');
            if (workerRef.current) {
                workerRef.current.postMessage({ type: 'STOP' });
                workerRef.current.terminate();
                workerRef.current = null;
            }
        };
    }, []);

    // 3. Visual Handshake (Regime-Aware Background)
    const currentRegime = useSystemStore((state) => state.currentRegime);

    useEffect(() => {
        const body = document.body;
        // Base transition
        body.style.transition = 'background-color 2s ease, box-shadow 2s ease';

        switch (currentRegime) {
            case 'LAMINAR':
                // Subtle Green Tint (via box-shadow inset to avoid overriding bg-hud too harshly)
                body.style.boxShadow = 'inset 0 0 100px rgba(16, 185, 129, 0.05)'; // emerald-500
                break;
            case 'TURBULENT':
                // Amber/Orange Tint
                body.style.boxShadow = 'inset 0 0 150px rgba(245, 158, 11, 0.1)'; // amber-500
                break;
            case 'DECOHERENT':
                // Red/Crimson Tint + slight pulse effect ideally, but static for now
                body.style.boxShadow = 'inset 0 0 200px rgba(239, 68, 68, 0.15)'; // red-500
                break;
                break;
        }
    }, [currentRegime]);

    // 4. Forensic Scrub Hook (Directive-106)
    const { isReplaying, replayTime, startTime, endTime } = useSystemStore(state => state.replay);

    useEffect(() => {
        if (!workerRef.current) return;

        if (isReplaying) {
            // Trigger Replay Mode
            workerRef.current.postMessage({
                type: 'START_REPLAY',
                payload: {
                    symbol: 'BTC-USDT', // Default active symbol
                    startTime: startTime || Date.now() - 60000, // Default to last minute if undefined
                    endTime: endTime || Date.now()
                }
            });
        } else {
            // Return to Live
            workerRef.current.postMessage({
                type: 'STOP_REPLAY'
            });
        }
    }, [isReplaying, startTime, endTime]);

    useEffect(() => {
        if (isReplaying && replayTime && workerRef.current) {
            workerRef.current.postMessage({
                type: 'SCRUB_SEEK',
                payload: { timestamp: replayTime }
            });
        }
    }, [replayTime, isReplaying]);

    return <>{children}</>;
}
