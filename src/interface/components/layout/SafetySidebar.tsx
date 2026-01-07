'use client';

import { usePortfolioStore } from '@/lib/stores/portfolio-store';
import { cn } from '@/lib/utils';
import { Shield, ShieldAlert, Cpu } from 'lucide-react';

const RISKS = [
    { id: 0, level: 'Q0', lots: 0.01, label: 'MIN' },
    { id: 1, level: 'Q1', lots: 0.05, label: '' },
    { id: 2, level: 'Q2', lots: 0.10, label: 'STD' },
    { id: 3, level: 'Q3', lots: 0.25, label: '' },
    { id: 4, level: 'Q4', lots: 0.50, label: 'AGG' },
    { id: 5, level: 'MAX', lots: 1.00, label: 'MAX' },
];

export function SafetySidebar() {
    // We bind to the store to read/write manual risk caps
    const manualRiskCap = usePortfolioStore((s) => s.manualRiskCap);
    const setRiskCap = usePortfolioStore((s) => s.setRiskCap);
    const systemRiskTier = usePortfolioStore((s) => s.systemRiskTier);

    return (
        <aside className="w-[64px] h-full border-r border-white/10 bg-[#0D1117] flex flex-col items-center py-4 gap-4 hidden md:flex">
            {/* Icon */}
            <div className="p-2 bg-white/5 rounded-md text-white/60">
                <Shield className="w-5 h-5" />
            </div>

            {/* Vertical Tiered Position Scaling */}
            <div className="flex-1 flex flex-col-reverse justify-center gap-1 w-full px-2">
                {RISKS.map((risk) => {
                    const isCap = manualRiskCap === risk.id;
                    const isBelowCap = manualRiskCap !== null && risk.id <= manualRiskCap;
                    const isActive = systemRiskTier === risk.id;

                    return (
                        <button
                            key={risk.level}
                            onClick={() => setRiskCap(isCap ? null : risk.id)} // Toggle off if clicked again
                            className={cn(
                                "w-full h-8 rounded flex items-center justify-center relative transition-all duration-200 group",
                                isCap ? "bg-red-500/20 ring-1 ring-red-500" :
                                    isBelowCap ? "bg-white/5 hover:bg-white/10" : "bg-transparent opacity-30"
                            )}
                        >
                            {/* Hover Label */}
                            <div className="absolute right-full mr-2 px-2 py-1 bg-black border border-white/10 rounded text-[10px] whitespace-nowrap hidden group-hover:block z-50">
                                {risk.level} | {risk.lots} lots
                            </div>

                            {/* Bar Visual */}
                            <div
                                className={cn(
                                    "w-1.5 rounded-full transition-all duration-300",
                                    isCap ? "bg-red-500 h-4" :
                                        isActive ? "bg-purple-500 h-6 w-2 shadow-lg shadow-purple-500/50" :
                                            "bg-white/20 h-2"
                                )}
                            />
                        </button>
                    );
                })}
            </div>

            {/* Bottom Status */}
            <div className="p-2 bg-white/5 rounded-md text-white/60" title="System Auto-Pilot">
                <Cpu className="w-5 h-5" />
            </div>
        </aside>
    );
}
