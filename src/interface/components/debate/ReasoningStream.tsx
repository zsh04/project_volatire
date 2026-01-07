'use client';

import { useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Brain, Eraser, Eye, Activity } from 'lucide-react';
import { useSystemStore } from '@/lib/stores/system-store';

/**
 * Directive-UX: Cognitive Pane (Reasoning Stream)
 * 
 * Features:
 * - Ghosting: Nullified thoughts fade out with a blur (History of Erasure)
 * - Live Feed: Stream of consciousness from agents
 */

export function ReasoningStream() {
    const thoughts = useSystemStore((state) => state.reasoningTrace);
    const scrollRef = useRef<HTMLDivElement>(null);

    // Auto-scroll logic
    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [thoughts]);

    return (
        <div className="h-full w-full flex flex-col font-mono text-xs overflow-hidden" ref={scrollRef}>
            <div className="flex-1 overflow-y-auto p-4 space-y-3 pb-20">
                <AnimatePresence>
                    {thoughts.map((thought) => (
                        <motion.div
                            key={thought.id}
                            initial={{ opacity: 0, x: 20 }}
                            animate={{
                                opacity: thought.status === 'nullified' ? 0.7 : 1,
                                x: 0,
                                filter: thought.status === 'nullified' ? 'grayscale(100%) blur(0.5px)' : 'none'
                            }}
                            exit={{ opacity: 0 }}
                            className={`relative pl-3 border-l-2 group ${thought.status === 'nullified' ? 'border-red-500/70 text-red-400' :
                                    thought.status === 'consensus' ? 'border-emerald-500 text-white' :
                                        'border-cyan-500/50 text-cyan-200'
                                }`}
                        >
                            <div className="flex items-center gap-2 mb-1 opacity-90 justify-between">
                                <div className="flex items-center gap-2">
                                    <span className={`font-bold ${thought.status === 'nullified' ? 'text-red-500' :
                                            thought.status === 'consensus' ? 'text-emerald-400' : 'text-cyan-400'
                                        }`}>{thought.author}</span>
                                    <span className="text-[10px] text-white/40">{new Date(thought.timestamp).toLocaleTimeString()}</span>
                                    {thought.status === 'nullified' && <Eraser size={10} className="text-red-500" />}
                                </div>
                                {thought.status !== 'nullified' && (
                                    <button
                                        onClick={() => console.log(`🔍 INSPECT TRUTH ENVELOPE [${thought.id}]`)}
                                        className="opacity-100 text-[10px] border border-white/20 px-2 py-0.5 rounded hover:bg-white/10 hover:border-white/50 hover:text-white flex items-center gap-1 transition-all"
                                    >
                                        <Eye size={10} /> INSPECT
                                    </button>
                                )}
                            </div>
                            <p className="leading-relaxed opacity-100">{thought.content}</p>
                        </motion.div>
                    ))}
                </AnimatePresence>

                {thoughts.length === 0 && (
                    <div className="text-center text-white/20 mt-20 flex flex-col items-center gap-2">
                        <Activity size={24} className="animate-pulse" />
                        <span>AWAITING SYNAPSE IGNITION...</span>
                    </div>
                )}

                {/* Sanity Watermark */}
                <div className="fixed bottom-32 right-8 pointer-events-none opacity-5">
                    <Brain size={120} />
                </div>
            </div>
        </div>
    );
}
