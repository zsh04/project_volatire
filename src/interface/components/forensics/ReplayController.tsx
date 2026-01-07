import React, { useState, useEffect, useRef } from 'react';
import { useSystemStore } from '@/lib/stores/system-store';

export function ReplayController() {
    const isReplaying = useSystemStore(state => state.replay.isReplaying);
    const replayTime = useSystemStore(state => state.replay.replayTime);
    const setReplay = useSystemStore(state => state.setReplay);

    // Local state for scrubber dragging
    const [scrubbing, setScrubbing] = useState(false);
    const [localTime, setLocalTime] = useState(0);

    // Sync local time with store when not scrubbing
    useEffect(() => {
        if (!scrubbing) {
            setLocalTime(replayTime);
        }
    }, [replayTime, scrubbing]);

    const handleScrub = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = Number(e.target.value);
        setLocalTime(val);
        setScrubbing(true);
    };

    const handleScrubEnd = () => {
        setScrubbing(false);
        // Dispatch "Jump" to time
        setReplay({ replayTime: localTime });
    };

    if (!isReplaying) return null;

    return (
        <div className="absolute bottom-12 left-1/2 -translate-x-1/2 w-[600px] h-16 bg-black/80 backdrop-blur-md border border-amber-500/30 rounded-full flex items-center px-6 gap-4 z-50 shadow-[0_0_30px_rgba(245,158,11,0.2)] animate-in fade-in slide-in-from-bottom-4">
            <div className="text-amber-500 font-mono text-xs whitespace-nowrap animate-pulse">
                🔴 REPLAY MODE
            </div>

            <button
                className="text-amber-500 hover:text-amber-400"
                onClick={() => setReplay({ isReplaying: false })}
            >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>

            <input
                type="range"
                min={0} // Placeholder: Will need session StartTime
                max={100} // Placeholder: Will need session EndTime
                value={localTime}
                onChange={handleScrub}
                onMouseUp={handleScrubEnd}
                onTouchEnd={handleScrubEnd}
                className="w-full h-1 bg-white/10 rounded-lg appearance-none cursor-pointer accent-amber-500"
            />

            <div className="font-mono text-amber-500 text-xs w-20 text-right">
                {new Date(localTime).toLocaleTimeString().split(' ')[0]}
            </div>
        </div>
    );
}
