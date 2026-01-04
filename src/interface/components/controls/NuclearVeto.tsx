'use client';

import { useState } from 'react';
import { usePortfolioStore } from '@/lib/stores/portfolio-store';
import { useSystemStore } from '@/lib/stores/system-store';

export function NuclearVeto() {
    const [isArming, setIsArming] = useState(false);
    const [isArmed, setIsArmed] = useState(false);

    // Simulation of arming process (hold to arm?)
    // For now, simple click with confirmation state

    const { isHalted, setHalted } = useSystemStore();

    const handleClick = async () => {
        if (isHalted) return; // Already halted

        if (!isArmed) {
            setIsArmed(true);
            setTimeout(() => setIsArmed(false), 3000); // Reset after 3s if not confirmed
        } else {
            // Trigger Veto
            try {
                const res = await fetch('/api/halt', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ timestamp: Date.now() })
                });

                if (res.ok) {
                    setHalted(true);
                    console.log('☢️ SYSTEM KILLED BY USER ☢️');
                }
            } catch (e) {
                console.error('Halt failed', e);
            }
            setIsArmed(false);
        }
    };

    return (
        <button
            onClick={handleClick}
            className={`
        relative overflow-hidden transition-all duration-300
        flex items-center justify-center gap-2
        ${isArmed
                    ? 'bg-red-600 hover:bg-red-700 w-48'
                    : 'bg-red-900/30 hover:bg-red-900/50 w-12 hover:w-48 group'
                }
        h-10 rounded text-white font-bold border border-red-500/50 backdrop-blur-md
      `}
        >
            <span className="text-lg">⚠️</span>
            <span className={`
        whitespace-nowrap transition-all duration-300
        ${isArmed || 'group-hover:w-auto overflow-hidden w-0 opacity-0 group-hover:opacity-100'}
      `}>
                {isHalted ? 'SYSTEM HALTED' : (isArmed ? 'CONFIRM VETO' : 'NUCLEAR VETO')}
            </span>

            {/* Progress bar for arming (visual flair) */}
            {isArmed && (
                <div className="absolute bottom-0 left-0 h-1 bg-white animate-[width_3s_linear_reverse_forwards]" />
            )}
        </button>
    );
}
