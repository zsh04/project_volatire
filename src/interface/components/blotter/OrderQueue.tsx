import React, { useState } from 'react';
import { useSystemStore } from '@/lib/stores/system-store';
import { formatDistanceToNow } from 'date-fns';
import { Filter, XCircle, CheckCircle, Clock } from 'lucide-react';

/**
 * Directive-105: Order Queue (Tactical Manager)
 * Manages working orders and trade flags.
 */
export function OrderQueue() {
    const finance = useSystemStore((state) => state.finance);
    const { orders = [] } = finance; // Default empty

    // UI State for "Maker Only" Toggle (Mock for now, normally D-106)
    const [makerOnly, setMakerOnly] = useState(true);

    const handleCancel = (id: string) => {
        console.log(`[TACTICAL] CANCEL ORDER: ${id}`);
        // TODO: Wire to RPC
    };

    const handleCancelAll = () => {
        console.log(`[TACTICAL] CANCEL ALL`);
    };

    return (
        <div className="h-full flex flex-col bg-[#0D1117] text-xs font-mono relative overflow-hidden">
            {/* Scanline Effect */}
            <div className="absolute inset-0 pointer-events-none bg-[url('/scanlines.png')] opacity-5 z-0"></div>

            <div className="h-6 flex items-center justify-between px-4 bg-white/5 border-b border-white/5 text-white/40 tracking-wider font-bold z-10">
                <span>ORDER QUEUE</span>
                <div className="flex items-center gap-4">
                    {/* Maker Only Toggle */}
                    <button
                        onClick={() => setMakerOnly(!makerOnly)}
                        className={`flex items-center gap-1.5 px-2 py-0.5 rounded transition-all ${makerOnly ? 'bg-amber-500/20 text-amber-400' : 'bg-white/5 text-white/40'}`}
                    >
                        <Filter size={10} />
                        <span className="text-[9px]">MAKER-ONLY</span>
                    </button>
                    {/* Flatten/Cancel All */}
                    <button
                        onClick={handleCancelAll}
                        className="text-[9px] hover:text-white transition-colors flex items-center gap-1"
                    >
                        <span>KILL ALL</span>
                    </button>
                </div>
            </div>

            {/* Header */}
            <div className="grid grid-cols-6 px-4 py-2 border-b border-white/5 text-white/30 uppercase tracking-wide z-10 bg-[#0D1117]">
                <div className="col-span-1">ID</div>
                <div>Side</div>
                <div className="text-right">Qty</div>
                <div className="text-right">Price</div>
                <div className="text-right">Status</div>
                <div className="text-center">Act</div>
            </div>

            {/* Data Rows */}
            <div className="flex-1 overflow-y-auto z-10 scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                {orders.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-white/20 italic">
                        NO WORKING ORDERS
                    </div>
                ) : (
                    orders.map((o) => {
                        const isBuy = o.side.toUpperCase() === 'BUY';
                        const sideColor = isBuy ? 'text-emerald-400' : 'text-rose-400';

                        return (
                            <div key={o.orderId} className="grid grid-cols-6 px-4 py-2 border-b border-white/5 hover:bg-white/5 transition-colors cursor-pointer group">
                                <div className="col-span-1 font-bold text-white/60 truncate" title={o.orderId}>
                                    {o.orderId.substring(0, 6)}...
                                </div>
                                <div className={`font-bold ${sideColor}`}>
                                    {o.side.toUpperCase()}
                                </div>
                                <div className="text-right text-white/80">
                                    {o.quantity.toFixed(4)}
                                </div>
                                <div className="text-right font-mono text-white/60">
                                    {o.limitPrice.toFixed(2)}
                                </div>
                                <div className="text-right flex justify-end items-center gap-1">
                                    {o.status === 'OPEN' && <Clock size={10} className="text-amber-500" />}
                                    <span className="text-[10px] text-white/40">{o.status}</span>
                                </div>
                                <div className="text-center flex justify-center">
                                    <button
                                        onClick={(e) => { e.stopPropagation(); handleCancel(o.orderId); }}
                                        className="opacity-0 group-hover:opacity-100 text-rose-500 hover:text-rose-400 transition-opacity"
                                    >
                                        <XCircle size={12} />
                                    </button>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>

            {/* Footer Info */}
            <div className="h-6 flex items-center justify-between px-4 bg-white/5 border-t border-white/5 text-[9px] text-white/30 z-10">
                <span>Working: {orders.filter(o => o.status === 'OPEN').length}</span>
                <span>Latency: -- ms</span>
            </div>
        </div>
    );
}
