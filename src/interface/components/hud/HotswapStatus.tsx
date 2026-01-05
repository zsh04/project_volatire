import React from 'react';
import { useSystemStore } from '@/lib/stores/system-store';
import { Activity, RefreshCw } from 'lucide-react';

export const HotswapStatus: React.FC = () => {
    const isHotswap = useSystemStore(state => state.hotswapActive);

    if (!isHotswap) return null;

    return (
        <div className="bg-amber-500/10 backdrop-blur-md border border-amber-500/50 rounded-md p-2 flex items-center gap-3 animate-pulse">
            <RefreshCw className="w-4 h-4 text-amber-500 animate-spin" />
            <div className="flex flex-col">
                <span className="text-[10px] font-bold text-amber-500 uppercase tracking-widest">
                    Hot-Swap Protocol
                </span>
                <span className="text-[10px] text-amber-300/80 font-mono">
                    Shadow Mode Verification...
                </span>
            </div>
            {/* Progress Bar simulation */}
            <div className="h-1 w-16 bg-amber-900/50 rounded-full overflow-hidden ml-2">
                <div className="h-full bg-amber-500 animate-[width_2s_ease-in-out_infinite]" style={{ width: '60%' }} />
            </div>
        </div>
    );
};
