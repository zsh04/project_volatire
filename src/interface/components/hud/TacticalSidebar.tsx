"use client";

import React from 'react';
import { useSystemStore } from '@/lib/stores/system-store';
import { usePortfolioStore } from '@/lib/stores/portfolio-store';
import { cn } from '@/lib/utils';
import { Shield, TrendingUp, TrendingDown, PauseCircle, Activity, Zap, Lock, Cpu } from 'lucide-react';
import { motion } from 'framer-motion';
import * as reflexClient from '@/lib/grpc/reflex-client';



const RISKS = [
    { id: 0, level: 'Q0', lots: 0.01, label: 'MIN' },
    { id: 1, level: 'Q1', lots: 0.05, label: '' },
    { id: 2, level: 'Q2', lots: 0.10, label: 'STD' },
    { id: 3, level: 'Q3', lots: 0.25, label: '' },
    { id: 4, level: 'Q4', lots: 0.50, label: 'AGG' },
    { id: 5, level: 'MAX', lots: 1.00, label: 'MAX' },
];


export function TacticalSidebar() {
    const { legislation, updateLegislation } = useSystemStore();
    const manualRiskCap = usePortfolioStore((s) => s.manualRiskCap);
    const setRiskCap = usePortfolioStore((s) => s.setRiskCap);
    const systemRiskTier = usePortfolioStore((s) => s.systemRiskTier);


    // D-107: RPC Integration
    const syncLegislation = async (updates: Partial<typeof legislation> & { snapToBreakeven?: boolean }) => {
        const merged = { ...legislation, ...updates };
        try {
            await reflexClient.updateLegislation(
                merged.bias,
                merged.aggression,
                merged.makerOnly,
                merged.hibernation,
                updates.snapToBreakeven || false
            );
        } catch (e) {
            console.error("Failed to sync legislation:", e);
        }
    };

    const handleBiasChange = (bias: 'NEUTRAL' | 'LONG_ONLY' | 'SHORT_ONLY') => {
        updateLegislation({ bias });
        syncLegislation({ bias });
    };

    const handleToggle = (field: 'makerOnly' | 'hibernation') => {
        const newValue = !legislation[field];
        updateLegislation({ [field]: newValue });
        syncLegislation({ [field]: newValue });
    };

    const handleAggressionChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = parseFloat(e.target.value);
        updateLegislation({ aggression: val });
        // Debounce happens naturally by user stopping adjustment or can add explicit debounce if needed
        // For slider, we might want to sync onMouseUp, but for now syncing on change is acceptable with rate limiting if needed
        syncLegislation({ aggression: val });
    };

    const handleSnap = () => {
        console.log("SNAP TO BREAK EVEN TRIGGERED");
        syncLegislation({ snapToBreakeven: true });
    };

    return (
        <div className="flex flex-col gap-4 p-4 border-l border-white/10 bg-black/40 backdrop-blur-md h-full w-[240px]">
            <div className="flex items-center gap-2 mb-2">
                <Shield className="w-5 h-5 text-emerald-400" />
                <h2 className="text-sm font-mono tracking-widest text-emerald-400">TACTICAL OPS</h2>
            </div>

            {/* STRATEGIC BIAS */}
            <div className="space-y-2">
                <label className="text-xs text-zinc-500 font-mono">STRATEGIC BIAS</label>
                <div className="grid grid-cols-3 gap-1">
                    <button
                        onClick={() => handleBiasChange('LONG_ONLY')}
                        aria-label="Set bias to Long Only"
                        title="Long Only Bias"
                        aria-pressed={legislation.bias === 'LONG_ONLY'}
                        className={`p-2 flex justify-center items-center border border-zinc-800 rounded hover:border-emerald-500/50 transition-colors ${legislation.bias === 'LONG_ONLY' ? 'bg-emerald-900/30 border-emerald-500 text-emerald-400' : 'text-zinc-600'}`}
                    >
                        <TrendingUp className="w-4 h-4" />
                    </button>
                    <button
                        onClick={() => handleBiasChange('NEUTRAL')}
                        aria-label="Set bias to Neutral"
                        title="Neutral Bias"
                        aria-pressed={legislation.bias === 'NEUTRAL'}
                        className={`p-2 flex justify-center items-center border border-zinc-800 rounded hover:border-emerald-500/50 transition-colors ${legislation.bias === 'NEUTRAL' ? 'bg-zinc-800 border-zinc-600 text-zinc-300' : 'text-zinc-600'}`}
                    >
                        <Activity className="w-4 h-4" />
                    </button>
                    <button
                        onClick={() => handleBiasChange('SHORT_ONLY')}
                        aria-label="Set bias to Short Only"
                        title="Short Only Bias"
                        aria-pressed={legislation.bias === 'SHORT_ONLY'}
                        className={`p-2 flex justify-center items-center border border-zinc-800 rounded hover:border-red-500/50 transition-colors ${legislation.bias === 'SHORT_ONLY' ? 'bg-red-900/30 border-red-500 text-red-400' : 'text-zinc-600'}`}
                    >
                        <TrendingDown className="w-4 h-4" />
                    </button>
                </div>
                <div className="text-[10px] text-center font-mono text-zinc-500">
                    {legislation.bias} MODE
                </div>
            </div>

            {/* RISK AGGRESSION */}
            <div className="space-y-2 pt-2 border-t border-white/5">
                <label className="text-xs text-zinc-500 font-mono flex justify-between">
                    <span>AGGRESSION</span>
                    <span className="text-emerald-400">{legislation.aggression.toFixed(1)}x</span>
                </label>
                <input
                    type="range"
                    min="0.1"
                    max="2.0"
                    step="0.1"
                    value={legislation.aggression}
                    onChange={handleAggressionChange}
                    className="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-emerald-500"
                />
            </div>

            {/* SAFETY / RISK CAPS */}
            <div className="mt-4 pt-4 border-t border-white/5 space-y-2 flex-1 flex flex-col">
                <div className="flex items-center gap-2 mb-2 text-zinc-500">
                    <Shield className="w-4 h-4" />
                    <label className="text-xs font-mono">RISK CEILING</label>
                </div>
                <div className="flex-1 flex flex-col-reverse justify-center gap-1 w-full px-2">
                    {RISKS.map((risk) => {
                        const isCap = manualRiskCap === risk.id;
                        const isBelowCap = manualRiskCap !== null && risk.id <= manualRiskCap;
                        const isActive = systemRiskTier === risk.id;

                        return (
                            <button
                                key={risk.level}
                                onClick={() => setRiskCap(isCap ? null : risk.id)} // Toggle off if clicked again
                                aria-label={`Set risk cap to ${risk.level}`}
                                aria-pressed={isCap}
                                className={cn(
                                    "w-full h-8 rounded flex items-center justify-between px-2 relative transition-all duration-200 group border border-transparent",
                                    isCap ? "bg-red-500/20 border-red-500" :
                                        isBelowCap ? "bg-white/5 hover:bg-white/10" : "bg-transparent opacity-30"
                                )}
                            >
                                <span className={cn("text-[10px] font-mono", isCap ? "text-red-400" : "text-zinc-500")}>
                                    {risk.level}
                                </span>

                                {/* Bar Visual */}
                                <div
                                    className={cn(
                                        "h-1.5 rounded-full transition-all duration-300",
                                        isCap ? "bg-red-500 w-16" :
                                            isActive ? "bg-purple-500 w-full shadow-lg shadow-purple-500/50" :
                                                "bg-white/20 w-8"
                                    )}
                                />
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* TOGGLES */}
            <div className="space-y-2 pt-2 border-t border-white/5">
                <button
                    onClick={() => handleToggle('makerOnly')}
                    aria-pressed={legislation.makerOnly}
                    className={`w-full flex items-center justify-between p-2 rounded border transition-colors ${legislation.makerOnly ? 'border-amber-500/50 bg-amber-900/10 text-amber-400' : 'border-zinc-800 text-zinc-500 hover:border-zinc-700'}`}
                >
                    <span className="text-xs font-mono">MAKER ONLY</span>
                    <Lock className="w-3 h-3" />
                </button>

                <button
                    onClick={() => handleToggle('hibernation')}
                    aria-pressed={legislation.hibernation}
                    className={`w-full flex items-center justify-between p-2 rounded border transition-colors ${legislation.hibernation ? 'border-purple-500/50 bg-purple-900/10 text-purple-400' : 'border-zinc-800 text-zinc-500 hover:border-zinc-700'}`}
                >
                    <span className="text-xs font-mono">HIBERNATE</span>
                    <PauseCircle className="w-3 h-3" />
                </button>
            </div>

            {/* COMMANDS */}
            <div className="mt-auto pt-4 border-t border-white/5">
                <button
                    onClick={handleSnap}
                    className="w-full group relative overflow-hidden p-2 border border-zinc-800 hover:border-emerald-500/50 rounded transition-all active:scale-95"
                >
                    <div className="absolute inset-0 bg-emerald-500/5 group-hover:bg-emerald-500/10 transition-colors" />
                    <div className="flex items-center justify-center gap-2 text-zinc-400 group-hover:text-emerald-400">
                        <Zap className="w-3 h-3" />
                        <span className="text-xs font-mono font-bold">SNAP TO BREAK-EVEN</span>
                    </div>
                </button>
            </div>
        </div>
    );
}
