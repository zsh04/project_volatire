'use client';

import { useAgentStore } from '@/lib/stores/agent-store';
import { useEffect, useState } from 'react';

export function ConsensusMeter() {
    const consensusScore = useAgentStore((s) => s.consensusScore);
    const simons = useAgentStore((s) => s.simons);
    const hypatia = useAgentStore((s) => s.hypatia);

    // Color gradient from red (low) to green (high)
    const getConsensusColor = (score: number) => {
        if (score < 0.3) return 'bg-red-500';
        if (score < 0.7) return 'bg-yellow-500';
        return 'bg-laminar';
    };

    return (
        <div className="space-y-4">
            <div>
                <div className="flex justify-between text-sm mb-2">
                    <span className="text-gray-400">Consensus</span>
                    <span className="font-mono">{(consensusScore * 100).toFixed(1)}%</span>
                </div>

                {/* Consensus bar */}
                <div className="w-full h-3 bg-gray-800 rounded-full overflow-hidden">
                    <div
                        className={`h-full transition-all duration-300 ${getConsensusColor(consensusScore)}`}
                        style={{ width: `${consensusScore * 100}%` }}
                    />
                </div>
            </div>

            {/* Persona status */}
            <div className="grid grid-cols-2 gap-4 text-xs">
                <div className="space-y-1">
                    <div className="flex items-center gap-2">
                        <div className={`w-2 h-2 rounded-full ${simons.status === 'active' ? 'bg-green-500 animate-pulse' :
                                simons.status === 'degraded' ? 'bg-yellow-500' :
                                    'bg-gray-500'
                            }`} />
                        <span className="font-bold">SIMONS</span>
                    </div>
                    <p className="text-gray-500 line-clamp-2">{simons.lastReason}</p>
                </div>

                <div className="space-y-1">
                    <div className="flex items-center gap-2">
                        <div className={`w-2 h-2 rounded-full ${hypatia.status === 'active' ? 'bg-green-500 animate-pulse' :
                                hypatia.status === 'degraded' ? 'bg-yellow-500' :
                                    'bg-gray-500'
                            }`} />
                        <span className="font-bold">HYPATIA</span>
                    </div>
                    <p className="text-gray-500 line-clamp-2">{hypatia.lastReason}</p>
                </div>
            </div>
        </div>
    );
}
