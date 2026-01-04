'use client';

import { useState, useEffect } from 'react';
import { clsx } from 'clsx';
import { SystemStatusPanel } from './SystemStatusPanel';
import { MarketInternals } from '../hud/MarketInternals';
import { SafetyStaircase } from '../hud/SafetyStaircase';

// Placeholder for L2 Stream (Directive-72: Eradicate Mocks)
function OrderBookPlaceholder() {
    return (
        <div className="flex flex-col h-full bg-[#0D1117] relative">
            <div className="grid grid-cols-3 p-2 text-white/40 border-b border-white/5 text-[10px] font-mono">
                <span>SIZE</span>
                <span className="text-center">PRICE</span>
                <span className="text-right">TOTAL</span>
            </div>
            <div className="flex-1 flex items-center justify-center text-white/20 text-xs font-mono animate-pulse">
                WAITING FOR L2 FEED...
            </div>
        </div>
    );
}

export function DepthVault() {
    const [activeTab, setActiveTab] = useState<'market' | 'system'>('market');

    return (
        <div className="flex flex-col h-full w-full">
            {/* 1. Header with Tabs */}
            <div className="h-8 flex border-b border-white/5 bg-[#0D1117]">
                <button
                    onClick={() => setActiveTab('market')}
                    className={clsx(
                        "flex-1 text-[10px] font-mono font-bold tracking-wider hover:bg-white/5 transition-colors",
                        activeTab === 'market' ? "text-cyan-400 border-b-2 border-cyan-400" : "text-white/40"
                    )}
                >
                    MARKET DEPTH
                </button>
                <button
                    onClick={() => setActiveTab('system')}
                    className={clsx(
                        "flex-1 text-[10px] font-mono font-bold tracking-wider hover:bg-white/5 transition-colors",
                        activeTab === 'system' ? "text-purple-400 border-b-2 border-purple-400" : "text-white/40"
                    )}
                >
                    SYSTEM STATUS
                </button>
            </div>

            {/* 2. Content Area */}
            <div className="flex-1 overflow-hidden bg-[#0D1117] relative">
                {activeTab === 'market' ? (
                    <div className="h-full flex flex-col">
                        <div className="flex-none p-2 border-b border-white/5">
                            <MarketInternals />
                        </div>
                        <div className="flex-1 overflow-hidden">
                            <OrderBookPlaceholder />
                        </div>
                    </div>
                ) : (
                    <div className="h-full flex flex-col">
                        <div className="flex-none p-2 border-b border-white/5">
                            <SafetyStaircase />
                        </div>
                        <div className="flex-1 overflow-hidden">
                            <SystemStatusPanel />
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
