'use client';

import { useAgentStore } from '@/lib/stores/agent-store';
import { useEffect, useRef } from 'react';

export function ReasoningStream() {
    const reasoningStream = useAgentStore((s) => s.reasoningStream);
    const streamRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom on new entries
    useEffect(() => {
        if (streamRef.current) {
            streamRef.current.scrollTop = streamRef.current.scrollHeight;
        }
    }, [reasoningStream]);

    const getTypeColor = (type: 'info' | 'warning' | 'veto') => {
        switch (type) {
            case 'veto':
                return 'text-red-400 border-l-red-500';
            case 'warning':
                return 'text-yellow-400 border-l-yellow-500';
            default:
                return 'text-gray-400 border-l-gray-600';
        }
    };

    return (
        <div className="space-y-2">
            <h3 className="text-sm font-bold text-gray-500">REASONING STREAM</h3>

            <div
                ref={streamRef}
                className="h-64 overflow-y-auto space-y-2 scrollbar-thin scrollbar-thumb-gray-700"
            >
                {reasoningStream.length === 0 ? (
                    <p className="text-gray-600 text-sm italic">Awaiting decisions...</p>
                ) : (
                    reasoningStream.map((entry) => (
                        <div
                            key={entry.id}
                            className={`text-xs border-l-2 pl-3 py-1 ${getTypeColor(entry.type)}`}
                        >
                            <div className="flex items-center gap-2 mb-1">
                                <span className="font-bold">{entry.persona}:</span>
                                <span className="text-gray-600 font-mono text-[10px]">
                                    {new Date(entry.timestamp).toLocaleTimeString()}
                                </span>
                            </div>
                            <p>{entry.message}</p>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
}
