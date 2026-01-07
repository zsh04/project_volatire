import React, { useState } from 'react';
import { useSystemStore } from '@/lib/stores/system-store';

const IgnitionSwitch: React.FC = () => {
    const { ignitionStatus } = useSystemStore();
    const [isConfirming, setIsConfirming] = useState(false);

    const handleIgnite = async () => {
        if (ignitionStatus !== 'HIBERNATION') return;

        setIsConfirming(true);

        // Call the RPC
        try {
            const response = await fetch('/api/rpc/ignition', {
                method: 'POST',
            });
            if (!response.ok) throw new Error('Ignition request failed');

            // Optimistic Update
            useSystemStore.getState().setIgnitionStatus('WARMINGUP');
        } catch (err) {
            console.error('Failed to initiate ignition:', err);
        } finally {
            setTimeout(() => setIsConfirming(false), 2000);
        }
    };

    const gates = [
        { name: 'HARDWARE', status: ['HARDWARECHECK', 'WARMINGUP', 'PENNYTRADE', 'AWAITINGGEMMA', 'IGNITED'].includes(ignitionStatus) },
        { name: 'WARMUP', status: ['WARMINGUP', 'PENNYTRADE', 'AWAITINGGEMMA', 'IGNITED'].includes(ignitionStatus) },
        { name: 'PENNY TRADE', status: ['PENNYTRADE', 'AWAITINGGEMMA', 'IGNITED'].includes(ignitionStatus) },
        { name: 'GEMMA BLESSING', status: ['AWAITINGGEMMA', 'IGNITED'].includes(ignitionStatus) },
    ];

    const isIgnited = ignitionStatus === 'IGNITED';
    const inProgress = !['HIBERNATION', 'IGNITED'].includes(ignitionStatus);

    return (
        <div className="bg-black/40 backdrop-blur border border-white/10 rounded p-3 min-w-[220px]">
            <div className="flex items-center justify-between mb-2">
                <span className="text-[10px] font-mono font-bold text-white/60 tracking-wider">
                    IGNITION PROTOCOL
                </span>
                <div className={`w-2 h-2 rounded-full ${isIgnited ? 'bg-green-500 animate-pulse' : 'bg-amber-500'}`} />
            </div>

            {/* Status */}
            <div className="text-xs font-mono text-white mb-3">
                {ignitionStatus}
            </div>

            {/* Gates */}
            <div className="space-y-1 mb-3">
                {gates.map((gate, idx) => (
                    <div key={idx} className="flex items-center gap-2">
                        <div className={`w-1.5 h-1.5 rounded-full ${gate.status ? 'bg-green-500' : 'bg-white/20'}`} />
                        <span className={`text-[9px] font-mono ${gate.status ? 'text-green-400' : 'text-white/40'}`}>
                            {gate.name}
                        </span>
                    </div>
                ))}
            </div>

            {/* Ignite Button */}
            {!isIgnited && (
                <button
                    onClick={handleIgnite}
                    disabled={inProgress || isConfirming}
                    className={`
                        w-full py-2 px-3 rounded text-[10px] font-mono font-bold tracking-wider
                        ${inProgress || isConfirming
                            ? 'bg-amber-500/20 text-amber-400 cursor-wait'
                            : 'bg-red-600/80 hover:bg-red-500 text-white cursor-pointer'
                        }
                        transition-all duration-200
                        border border-white/10
                    `}
                >
                    {isConfirming ? 'INITIATING...' : inProgress ? 'IN PROGRESS...' : '🚀 IGNITE'}
                </button>
            )}

            {isIgnited && (
                <div className="text-center py-2 text-green-400 font-mono text-[10px] font-bold">
                    ✓ IGNITED
                </div>
            )}
        </div>
    );
};

export default IgnitionSwitch;
