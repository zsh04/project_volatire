import React, { useEffect, useRef, useState } from 'react';
import { useSystemStore } from '../../lib/stores/system-store';

const Vitality: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const { systemSanityScore, venue, currentRegime } = useSystemStore();

    // Synthesize Vitality from available metrics
    const vitality = {
        jitterUs: (1.0 - systemSanityScore) * 500, // Proxy: Lower sanity = higher jitter
        latencyUs: venue.rtt * 1000, // ms to us
        status: systemSanityScore > 0.8 ? 'OPTIMAL' : systemSanityScore > 0.5 ? 'DEGRADED' : 'CRITICAL'
    };


    // Waveform State
    const [dataPoints, setDataPoints] = useState<number[]>(new Array(100).fill(0));

    useEffect(() => {
        // Animation Loop for Waveform
        // In a real scenario, this would adhere to the 60Hz loop from Worker data.
        // For visualization, we simulate a pulse based on Jitter.

        let animationFrameId: number;
        let phase = 0;

        const render = () => {
            const canvas = canvasRef.current;
            if (!canvas) return;
            const ctx = canvas.getContext('2d');
            if (!ctx) return;

            // Clear
            ctx.clearRect(0, 0, canvas.width, canvas.height);

            // Jitter influence
            // 0-50us = Optimal (Green)
            // 50-100us = Degraded (Amber)
            // >100us = Critical (Red)
            const jitterRatio = Math.min(vitality.jitterUs / 100, 1.0); // 0.0 to 1.0

            // Color
            let strokeColor = '#10B981'; // Green-500
            if (vitality.status === 'DEGRADED') strokeColor = '#F59E0B'; // Amber-500
            if (vitality.status === 'CRITICAL') strokeColor = '#EF4444'; // Red-500

            // Draw Waveform
            ctx.beginPath();
            ctx.lineWidth = 1.5;
            ctx.strokeStyle = strokeColor;

            const amplitude = 15 + (jitterRatio * 20); // 15px base + jitter
            const frequency = 0.1 + (jitterRatio * 0.2);

            for (let x = 0; x < canvas.width; x++) {
                // Sine wave + Random noise (Jitter)
                const noise = (Math.random() - 0.5) * (jitterRatio * 10);
                const y = (canvas.height / 2) + Math.sin((x * frequency) + phase) * amplitude + noise;

                if (x === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            }

            ctx.stroke();

            // Update Phase
            phase += 0.2;
            animationFrameId = requestAnimationFrame(render);
        };

        render();

        return () => cancelAnimationFrame(animationFrameId);
    }, [vitality]);

    return (
        <div className="flex flex-col border border-white/10 bg-black/40 backdrop-blur-md rounded-md overflow-hidden p-3 w-[300px]">
            {/* Header */}
            <div className="flex justify-between items-center mb-2">
                <h3 className="text-xs font-mono text-gray-400 uppercase tracking-widest">Health & Latency Monitor</h3>
                <div className={`px-2 py-0.5 text-[10px] font-bold rounded-sm ${vitality.status === 'OPTIMAL' ? 'bg-green-900/50 text-green-400 border border-green-700/50' :
                    vitality.status === 'DEGRADED' ? 'bg-yellow-900/50 text-yellow-400 border border-yellow-700/50' :
                        'bg-red-900/50 text-red-500 border border-red-700/50 animate-pulse'
                    }`}>
                    {vitality.status}
                </div>
            </div>

            {/* Canvas (Heartbeat Ribbon) */}
            <div className="relative h-16 w-full bg-black/50 rounded border border-white/5 mb-3">
                <canvas
                    ref={canvasRef}
                    width={276}
                    height={64}
                    className="w-full h-full"
                />

                {/* Horizontal Threshold Line */}
                <div className="absolute top-1/2 left-0 right-0 h-[1px] bg-white/5 border-t border-dashed border-white/10" />
            </div>

            {/* Metrics Grid */}
            <div className="grid grid-cols-2 gap-2 text-[10px] font-mono">
                <div className="bg-white/5 p-2 rounded flex flex-col justify-between">
                    <span className="text-gray-500 mb-1">LATENCY</span>
                    <span className="text-l text-gray-200">{vitality.latencyUs.toFixed(0)} <span className="text-gray-600">us</span></span>
                </div>
                <div className="bg-white/5 p-2 rounded flex flex-col justify-between">
                    <span className="text-gray-500 mb-1">JITTER</span>
                    <span className={`text-l ${vitality.status === 'CRITICAL' ? 'text-red-500' : 'text-gray-200'}`}>
                        {vitality.jitterUs.toFixed(1)} <span className="text-gray-600">us</span>
                    </span>
                </div>
            </div>
        </div>
    );
};

export default Vitality;
