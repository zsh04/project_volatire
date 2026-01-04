import { motion } from 'framer-motion';
import { useSystemStore } from '@/lib/stores/system-store';

const TIERS = ['Q0', 'Q1', 'Q2', 'Q3', 'Q4', 'MAX'];

export function SafetyStaircase() {
    const { currentTier, nextTierProgress, isCooldown, vetoCount } = useSystemStore(state => state.staircase);

    const currentTierIndex = TIERS.indexOf(currentTier);
    const progressPercent = (nextTierProgress / 50) * 100;

    return (
        <div className="flex flex-col gap-2 p-3 bg-glass border border-glass-border/30 rounded-lg shadow-glow-sm backdrop-blur-md relative overflow-hidden group">
            {/* Ambient background glow */}
            <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />

            {/* Header */}
            <div className="flex justify-between items-center font-mono relative z-10">
                <span className="text-[10px] uppercase tracking-[0.2em] text-white/40">Safety Staircase</span>
                <span className={`text-xs font-bold ${isCooldown ? "text-red-500 animate-pulse text-glow-red" : "text-laminar text-glow-green"}`}>
                    {isCooldown ? `COOLDOWN (${vetoCount})` : currentTier}
                </span>
            </div>

            {/* XP Bar Container */}
            <div className="relative h-1.5 w-full bg-hud-dark rounded-full overflow-hidden border border-white/5 shadow-inner">
                {/* Progress Fill */}
                <motion.div
                    className={`absolute top-0 left-0 h-full ${isCooldown ? 'bg-red-500/50' : 'bg-laminar shadow-[0_0_10px_rgba(68,255,68,0.5)]'}`}
                    initial={{ width: 0 }}
                    animate={{ width: `${progressPercent}%` }}
                    transition={{ type: "spring", stiffness: 100, damping: 20 }}
                />
            </div>

            {/* Tier Indicators */}
            <div className="flex justify-between px-1 mt-0.5 relative z-10">
                {TIERS.map((tier, idx) => {
                    const isActive = idx <= currentTierIndex;
                    const isCurrent = idx === currentTierIndex;

                    return (
                        <div key={tier} className="flex flex-col items-center gap-1.5 group/tier">
                            <div
                                className={`w-1 h-1 rounded-full transition-all duration-300 
                                    ${isActive
                                        ? isCooldown ? 'bg-red-500 shadow-glow-red-sm' : 'bg-laminar shadow-glow-sm'
                                        : 'bg-white/10'
                                    } 
                                    ${isCurrent ? 'scale-150 animate-pulse' : ''}
                                `}
                            />
                            <span
                                className={`text-[8px] font-mono transition-colors duration-300 
                                    ${isActive
                                        ? 'text-white font-medium'
                                        : 'text-white/20'
                                    }
                                `}
                            >
                                {tier}
                            </span>
                        </div>
                    );
                })}
            </div>

            {/* Footer */}
            <div className="flex justify-between items-center text-[9px] font-mono mt-1 text-white/30 border-t border-white/5 pt-2 relative z-10">
                <span className="uppercase tracking-wider">Progress</span>
                <span className="tabular-nums font-medium text-white/50">
                    {nextTierProgress.toString().padStart(2, '0')} <span className="text-white/20">/</span> 50
                </span>
            </div>
        </div>
    );
}
