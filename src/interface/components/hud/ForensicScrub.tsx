'use client';

import { useEffect, useRef, useState } from 'react';
import { useSystemStore } from '../../lib/stores/system-store';
import { cn } from '../../lib/utils';

export function ForensicScrub() {
    const {
        scrubMode,
        setScrubMode,
        scrubTimestamp,
        setScrubTimestamp,
        forensicEvents
    } = useSystemStore();

    const [hoverTime, setHoverTime] = useState<number | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    // Timeline Config
    const WINDOW_MS = 60000; // 60s

    // Calculate current time relative to window
    // In live mode, "now" is the right edge. 
    // In scrub mode, "now" keeps moving away, but we want to anchor to the latest available data?
    // Actually, worker buffer is 60s. We should visualize "Now - 60s" to "Now".

    const [now, setNow] = useState(Date.now());

    useEffect(() => {
        if (!scrubMode) {
            const interval = setInterval(() => setNow(Date.now()), 100);
            return () => clearInterval(interval);
        }
    }, [scrubMode]);

    const handleScrub = (e: React.MouseEvent<HTMLDivElement>) => {
        if (!containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const width = rect.width;

        // Calculate percentage (0 = Oldest, 1 = Newest)
        const pct = Math.max(0, Math.min(1, x / width));

        // Convert to timestamp
        // Right edge is "now" (if live) or "frozen now" (if we want to lock it, but let's keep it simpler for now)
        // Let's assume the timeline is always [Now - 60s, Now]
        const targetTime = now - (WINDOW_MS * (1 - pct));

        setScrubMode(true);
        setScrubTimestamp(targetTime);
    };

    const handleExit = () => {
        setScrubMode(false);
        setScrubTimestamp(null);
    };

    return (
        <div className="absolute bottom-20 left-1/2 -translate-x-1/2 w-[600px] h-16 pointer-events-auto flex flex-col items-center gap-2">

            {/* Status Indicator */}
            <div className={cn(
                "px-3 py-1 text-xs font-mono rounded-full border backdrop-blur-md transition-all",
                scrubMode
                    ? "bg-red-500/20 border-red-500/50 text-red-100 animate-pulse"
                    : "bg-emerald-500/10 border-emerald-500/30 text-emerald-100/50 opacity-50 hover:opacity-100"
            )}>
                {scrubMode ? "🔴 FORENSIC REPLAY ACTIVE" : "🟢 LIVE FEED"}
            </div>

            {/* Timeline Track */}
            <div
                ref={containerRef}
                className="relative w-full h-8 bg-black/60 border border-slate-700/50 rounded-sm cursor-crosshair overflow-hidden group"
                onClick={handleScrub}
                onMouseMove={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    const x = e.clientX - rect.left;
                    const pct = x / rect.width;
                    setHoverTime(now - (WINDOW_MS * (1 - pct)));
                }}
                onMouseLeave={() => setHoverTime(null)}
            >
                {/* Grid Lines (Every 10s) */}
                {[0, 1, 2, 3, 4, 5].map(i => (
                    <div
                        key={i}
                        className="absolute top-0 bottom-0 bg-slate-800/50 w-px"
                        style={{ left: `${(i / 6) * 100}%` }}
                    />
                ))}

                {/* Event Markers */}
                {forensicEvents
                    .filter(e => e.timestamp > now - WINDOW_MS && e.timestamp <= now)
                    .map(e => {
                        const pct = 1 - ((now - e.timestamp) / WINDOW_MS);
                        return (
                            <div
                                key={e.id}
                                className={cn(
                                    "absolute top-0 h-full w-0.5 shadow-[0_0_10px_currentColor]",
                                    e.type === 'VETO' ? "bg-red-500 text-red-500" : "bg-yellow-500 text-yellow-500"
                                )}
                                style={{ left: `${pct * 100}%` }}
                            />
                        );
                    })
                }

                {/* Scrubber Head (If Active) */}
                {scrubMode && scrubTimestamp && (
                    <div
                        className="absolute top-0 bottom-0 w-0.5 bg-white shadow-[0_0_15px_white] z-10"
                        style={{ left: `${(1 - (now - scrubTimestamp) / WINDOW_MS) * 100}%` }}
                    />
                )}

                {/* Hover Head */}
                {hoverTime && !scrubMode && (
                    <div
                        className="absolute top-0 bottom-0 w-px bg-white/30 z-0"
                        style={{ left: `${(1 - (now - hoverTime) / WINDOW_MS) * 100}%` }}
                    />
                )}

                {/* Timestamp Label */}
                <div className="absolute top-1 right-2 text-[10px] text-slate-500 font-mono">
                    -{((now - (scrubTimestamp || hoverTime || now)) / 1000).toFixed(1)}s
                </div>
            </div>

            {/* Exit Button */}
            {scrubMode && (
                <button
                    onClick={handleExit}
                    className="mt-1 text-[10px] text-emerald-400 font-mono hover:text-emerald-300 underline"
                >
                    RETURN TO LIVE
                </button>
            )}
        </div>
    );
}
