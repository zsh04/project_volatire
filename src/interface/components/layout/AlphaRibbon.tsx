'use client';

import { Activity } from 'lucide-react';
import { NuclearVeto } from '../controls/NuclearVeto';
import { useSystemStore } from '@/lib/stores/system-store';

export function AlphaRibbon() {
    const { unrealizedPnl } = useSystemStore(state => state.finance);
    const pnlColor = unrealizedPnl >= 0 ? "text-emerald-400" : "text-red-500";
    const pnlSign = unrealizedPnl >= 0 ? "+" : "-";

    return (
        <header className="h-[48px] border-b border-white/10 bg-[#0D1117] flex items-center justify-between px-4 select-none sticky top-0 z-50">
            {/* Left: Identity */}
            <div className="flex items-center gap-3">
                <Activity className="w-5 h-5 text-yellow-400" />
                <h1 className="font-mono text-sm font-bold text-white tracking-widest uppercase">
                    Voltaire <span className="text-white/40">//</span> Pro
                </h1>
            </div>

            {/* Center: Global Metrics & Audit Loop */}
            <div className="hidden md:flex items-center gap-6 text-xs font-mono">
                <div className="flex items-center gap-2">
                    <span className="text-gray-500">PnL</span>
                    <span className={`${pnlColor} font-bold`}>
                        {pnlSign}${Math.abs(unrealizedPnl).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                    </span>
                </div>

                {/* Directive-66: Decay Ribbon */}
                <DecayRibbon />

                {/* Directive-67: Venue Health */}
                <VenueHealth />
            </div>

            {/* Right: Sovereign Control */}
            <div className="flex items-center gap-4">
                <div className="flex items-center gap-2 px-2 py-1 bg-white/5 rounded">
                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                    <span className="text-[10px] font-mono text-white/60">LIVE FEED</span>
                </div>
                <NuclearVeto />
            </div>
        </header>
    );
}

function DecayRibbon() {
    const { driftScore, isRecalibrating } = useSystemStore(state => state.audit);

    // Color mapping based on drift severity
    let colorClass = "text-emerald-400";
    if (driftScore > 0.15) colorClass = "text-yellow-400";
    if (driftScore > 0.30) colorClass = "text-red-500";

    return (
        <div className="flex items-center gap-2 min-w-[140px]">
            <span className="text-gray-500">MODEL DRIFT</span>
            {isRecalibrating ? (
                <span className="text-blue-400 animate-pulse">RECALIBRATING...</span>
            ) : (
                <div className="flex items-center gap-2">
                    <span className={colorClass}>{(driftScore * 100).toFixed(1)}%</span>
                    {/* Visual Bar */}
                    <div className="w-16 h-1.5 bg-white/10 rounded-full overflow-hidden">
                        <div
                            className={`h-full ${driftScore > 0.3 ? 'bg-red-500' : 'bg-emerald-400'} transition-all duration-500`}
                            style={{ width: `${Math.min(driftScore * 100 * 3, 100)}%` }} // Scale up for visibility
                        />
                    </div>
                </div>
            )}
        </div>
    );
}


function VenueHealth() {
    const { rtt, status } = useSystemStore(state => state.venue);

    // Status color mapping
    const colorMap = {
        'ONLINE': 'text-emerald-400',
        'DEGRADED': 'text-yellow-400',
        'OFFLINE': 'text-red-500 animate-pulse'
    };

    return (
        <div className="flex items-center gap-2 min-w-[120px]">
            <span className="text-gray-500">VENUE</span>
            <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${status === 'OFFLINE' ? 'bg-red-500' : (status === 'DEGRADED' ? 'bg-yellow-400' : 'bg-emerald-400')}`} />
                <span className={`text-xs font-mono ${colorMap[status]}`}>
                    {rtt}ms
                </span>
            </div>
        </div>
    );
}
