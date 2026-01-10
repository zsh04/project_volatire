import React, { useMemo } from 'react';
import { useSystemStore } from '@/lib/stores/system-store';
import { formatDistanceToNow } from 'date-fns';
import { closePosition } from '@/lib/grpc/reflex-client';

/**
 * Directive-105: Position Blotter (The Ledger)
 * Professional-grade position tracking.
 */
export function PositionBlotter() {
    // Using finance slice from system store (D-76)
    const finance = useSystemStore((state) => state.finance);
    const { positions = [] } = finance; // Default empty

    const totalAllocated = useMemo(() => {
        return positions.reduce((acc, p) => acc + (Math.abs(p.netSize) * p.avgEntryPrice), 0);
    }, [positions]);

    const handleFlatten = async (symbol: string) => {
        console.log(`[TACTICAL] FLATTEN REQUEST: ${symbol}`);
        try {
            await closePosition(symbol);
        } catch (err) {
            console.error('[TACTICAL] FLATTEN FAILED:', err);
        }
    };

    return (
        <div className="h-full flex flex-col bg-[#0D1117] text-xs font-mono relative overflow-hidden">
            {/* Scanline Effect */}
            <div className="absolute inset-0 pointer-events-none bg-[url('/scanlines.png')] opacity-5 z-0"></div>

            <div className="h-6 flex items-center justify-between px-4 bg-white/5 border-b border-white/5 text-white/40 tracking-wider font-bold z-10">
                <span>FISCAL DECK // BLOTTER</span>
                <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span className="text-emerald-500">LIVE</span>
                </div>
            </div>

            {/* Header Row */}
            <div className="grid grid-cols-7 px-4 py-2 border-b border-white/5 text-white/30 uppercase tracking-wide z-10 bg-[#0D1117]">
                <div className="col-span-1">Sym</div>
                <div className="text-right">Size</div>
                <div className="text-right">Entry</div>
                <div className="text-right">Mark</div>
                <div className="text-right">PnL</div>
                <div className="text-right">Time</div>
                <div className="text-center">Act</div>
            </div>

            {/* Data Rows */}
            <div className="flex-1 overflow-y-auto z-10 scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                {positions.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-white/20 italic">
                        NO ACTIVE EXPOSURE
                    </div>
                ) : (
                    positions.map((p) => {
                        const isProfit = p.unrealizedPnl >= 0;
                        const pnlColor = isProfit ? 'text-emerald-400' : 'text-rose-500';
                        const rowGlow = isProfit ? 'hover:bg-emerald-500/5' : 'hover:bg-rose-500/5 animate-pulse-slow';

                        return (
                            <div key={p.symbol} className={`grid grid-cols-7 px-4 py-2 border-b border-white/5 transition-colors cursor-pointer group ${rowGlow}`}>
                                <div className="col-span-1 font-bold text-white">{p.symbol}</div>
                                <div className="text-right text-white/80">{p.netSize?.toFixed(4)}</div>
                                <div className="text-right text-white/60">{p.avgEntryPrice?.toFixed(2)}</div>
                                <div className="text-right font-mono text-white/60">
                                    {p.currentPrice?.toFixed(2) || '--'}
                                </div>
                                <div className={`text-right font-bold ${pnlColor}`}>
                                    {isProfit ? '+' : ''}{p.unrealizedPnl?.toFixed(2)}
                                </div>
                                <div className="text-right text-white/40 text-[10px] pt-0.5">
                                    {formatDistanceToNow(p.entryTimestamp, { addSuffix: false })}
                                </div>
                                <div className="text-center flex justify-center">
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            useSystemStore.getState().setReplay({ isReplaying: true, startTime: p.entryTimestamp, endTime: Date.now() });
                                        }}
                                        className="opacity-0 group-hover:opacity-100 mr-2 p-1 hover:bg-amber-500/20 rounded transition-all"
                                        title="Replay Execution"
                                    >
                                        <svg className="w-3 h-3 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                    </button>
                                    <button
                                        onClick={(e) => { e.stopPropagation(); handleFlatten(p.symbol); }}
                                        className="opacity-0 group-hover:opacity-100 px-2 py-0.5 bg-rose-500/20 text-rose-400 hover:bg-rose-500 hover:text-white text-[9px] uppercase tracking-wider rounded transition-all"
                                    >
                                        FLATTEN
                                    </button>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>

            {/* Equity Guardian Footer */}
            <div className="mt-auto border-t border-white/10 p-2 grid grid-cols-3 bg-white/5 z-10 backdrop-blur-sm relative overflow-hidden">
                {/* Sparkline Background */}
                <div className="absolute inset-0 z-0 opacity-20 pointer-events-none">
                    <svg className="w-full h-full" preserveAspectRatio="none">
                        <polyline
                            points={finance.equityHistory?.map((val, i) => {
                                const min = Math.min(...(finance.equityHistory || []));
                                const max = Math.max(...(finance.equityHistory || []));
                                const range = max - min || 1;
                                const x = (i / ((finance.equityHistory?.length || 1) - 1)) * 100;
                                const y = 100 - ((val - min) / range) * 100;
                                return `${x},${y}`;
                            }).join(' ')}
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1"
                            className="text-emerald-500"
                        />
                    </svg>
                </div>

                <div className="flex flex-col z-10">
                    <span className="text-[9px] text-white/40 uppercase">Equity</span>
                    <span className="font-bold text-white md:text-sm">${finance.equity?.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                </div>
                <div className="flex flex-col items-center">
                    <span className="text-[9px] text-white/40 uppercase">Allocated</span>
                    {/* Placeholder for allocated calculation */}
                    <span className="font-mono text-white/80">${totalAllocated.toFixed(2)}</span>
                </div>
                <div className="flex flex-col items-end">
                    <span className="text-[9px] text-white/40 uppercase">Unr. PnL</span>
                    <span className={`font-bold md:text-sm ${finance.unrealizedPnl >= 0 ? 'text-emerald-400' : 'text-rose-500'}`}>
                        {finance.unrealizedPnl >= 0 ? '+' : ''}{finance.unrealizedPnl?.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </span>
                </div>
            </div>

            {/* Drawdown Warning Overlay (if strict mode) */}
            {/* Not implemented yet, logic in store */}
        </div>
    );
}
