'use client';

import { useSystemStore, SystemNode } from '@/lib/stores/system-store';
import { clsx } from 'clsx';
import { useMemo } from 'react';

function StatusLight({ status }: { status: SystemNode['status'] }) {
    const color = useMemo(() => {
        switch (status) {
            case 'online': return 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]';
            case 'optimizing': return 'bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.8)] animate-pulse';
            case 'degraded': return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]';
            case 'offline': return 'bg-slate-700';
            default: return 'bg-slate-700';
        }
    }, [status]);

    return (
        <div className={clsx("w-2 h-2 rounded-full", color)} />
    );
}

function NodeRow({ node }: { node: SystemNode }) {
    // Latency visualization (Simple bar)
    const latencyColor = node.latency < 50 ? 'bg-emerald-500' : node.latency < 200 ? 'bg-amber-400' : 'bg-red-500';
    const latencyWidth = Math.min((node.latency / 500) * 100, 100);

    return (
        <div className="flex items-center justify-between p-3 border-b border-white/5 hover:bg-white/5 transition-colors group cursor-default">

            {/* Left: ID & status */}
            <div className="flex items-center gap-3">
                <StatusLight status={node.status} />
                <div className="flex flex-col">
                    <span className="text-xs font-mono font-bold text-white tracking-widest">{node.name}</span>
                    <span className="text-[10px] uppercase text-white/40">{node.type}</span>
                </div>
            </div>

            {/* Middle: Metric */}
            <div className="flex flex-col items-end min-w-[80px]">
                <span className="text-sm font-mono text-cyan-400 font-bold">
                    {node.metricValue} <span className="text-[10px] text-cyan-400/50">{node.metricUnit}</span>
                </span>
                <span className="text-[9px] text-white/30 uppercase tracking-wider">{node.metricLabel}</span>
            </div>

            {/* Right: Latency (Hidden on tiny screens, visual indicator) */}
            <div className="w-12 flex flex-col items-end gap-1 ml-4">
                <div className="text-[9px] font-mono text-white/40">{node.latency}ms</div>
                <div className="w-full h-1 bg-white/10 rounded-full overflow-hidden">
                    <div
                        className={clsx("h-full rounded-full transition-all duration-300", latencyColor)}
                        style={{ width: `${latencyWidth}%` }}
                    />
                </div>
            </div>

        </div>
    );
}

export function SystemStatusPanel() {
    const nodes = useSystemStore(s => s.nodes);
    const nodeList = Object.values(nodes);

    return (
        <div className="flex flex-col h-full w-full overflow-y-auto custom-scrollbar">
            {nodeList.map((node) => (
                <NodeRow key={node.id} node={node} />
            ))}

            {/* Footer / Legend */}
            <div className="mt-auto p-4 border-t border-white/5">
                <div className="flex justify-between items-center text-[10px] text-white/30 font-mono">
                    <span>SYSTEM INTEGRITY</span>
                    <span className="text-emerald-500">99.9%</span>
                </div>
            </div>
        </div>
    );
}
