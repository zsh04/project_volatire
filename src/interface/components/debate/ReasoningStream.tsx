'use client';

import { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Brain, Eraser } from 'lucide-react';
import { useSystemStore } from '@/lib/stores/system-store';

/**
 * Directive-UX: Cognitive Pane (Reasoning Stream)
 * 
 * Features:
 * - Ghosting: Nullified thoughts fade out with a blur (History of Erasure)
 * - Live Feed: Stream of consciousness from agents
 */

interface Thought {
    id: string;
    agent: string;
    content: string;
    timestamp: number;
    status: 'active' | 'nullified' | 'consensus';
}

// Mock Data Generator (since we don't have the live gRPC stream fully hooked in for this demo)
const MOCK_THOUGHTS: Thought[] = [
    { id: '1', agent: 'SOROS', content: 'Market structure implies mean reversion on 1m timeframe.', timestamp: Date.now(), status: 'active' },
    { id: '2', agent: 'TALEB', content: 'Tail risk detected in derivative sizing. Vetoeing.', timestamp: Date.now(), status: 'nullified' },
    { id: '3', agent: 'GOVERNOR', content: 'Re-zeroing portfolio beta.', timestamp: Date.now(), status: 'consensus' },
];

export function ReasoningStream() {
    const [thoughts, setThoughts] = useState<Thought[]>(MOCK_THOUGHTS);
    const scrollRef = useRef<HTMLDivElement>(null);
    const systemSanityScore = useSystemStore((state) => state.systemSanityScore);

    // Mock Live Feed
    useEffect(() => {
        const interval = setInterval(() => {
            const agents = ['SOROS', 'TALEB', 'CHRONOS', 'GOVERNOR'];
            const contents = [
                'Checking latency thresholds...',
                'Volatility spike detected in sector 4.',
                'Adjusting alpha decay parameters.',
                'Optimizing execution route.',
                'Latency drift detected (45ms).',
                'Hotswap protocol engaged.'
            ];
            
            const newThought: Thought = {
                id: Date.now().toString(),
                agent: agents[Math.floor(Math.random() * agents.length)],
                content: contents[Math.floor(Math.random() * contents.length)],
                timestamp: Date.now(),
                status: Math.random() > 0.8 ? 'nullified' : Math.random() > 0.8 ? 'consensus' : 'active'
            };

            setThoughts(prev => [...prev.slice(-15), newThought]); // Keep last 15
        }, 3000);

        return () => clearInterval(interval);
    }, []);

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
                            animate={{ opacity: thought.status === 'nullified' ? 0.4 : 1, x: 0, filter: thought.status === 'nullified' ? 'blur(2px)' : 'none' }}
                            exit={{ opacity: 0 }}
                            className={`relative pl-3 border-l-2 ${
                                thought.status === 'nullified' ? 'border-red-500/50 text-red-500/50' : 
                                thought.status === 'consensus' ? 'border-emerald-500 text-white' : 
                                'border-blue-500/50 text-blue-200'
                            }`}
                        >
                            <div className="flex items-center gap-2 mb-1 opacity-70">
                                <span className="font-bold">{thought.agent}</span>
                                <span className="text-[10px]">{new Date(thought.timestamp).toLocaleTimeString()}</span>
                                {thought.status === 'nullified' && <Eraser size={10} />}
                            </div>
                            <p className="leading-relaxed opacity-90">{thought.content}</p>
                        </motion.div>
                    ))}
                </AnimatePresence>
                
                {/* Sanity Watermark */}
                <div className="fixed bottom-32 right-8 pointer-events-none opacity-10">
                    <Brain size={120} />
                </div>
            </div>
        </div>
    );
}
