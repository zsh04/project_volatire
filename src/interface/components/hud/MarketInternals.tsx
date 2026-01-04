'use client';

import { useMarketStore } from '@/lib/stores/market-store';

export function MarketInternals() {
    const vixCorrelation = useMarketStore((s) => s.vixCorrelation);
    const tickProxy = useMarketStore((s) => s.tickProxy);
    const alphaDecay = useMarketStore((s) => s.alphaDecay);

    const formatMetric = (value: number) => {
        if (value === 0) return '--';
        return value.toFixed(3);
    };

    const getCorrelationColor = (value: number) => {
        if (Math.abs(value) > 0.7) return 'text-red-400';
        if (Math.abs(value) > 0.4) return 'text-yellow-400';
        return 'text-green-400';
    };

    return (
        <div className="grid grid-cols-3 gap-4">
            {/* VIX Correlation */}
            <div className="text-center space-y-1">
                <p className="text-xs text-gray-500">VIX Correlation</p>
                <p className={`metric-value ${getCorrelationColor(vixCorrelation)}`}>
                    {formatMetric(vixCorrelation)}
                </p>
                <div className="h-1 w-full bg-gray-800 rounded-full overflow-hidden">
                    <div
                        className="h-full bg-red-500 transition-all"
                        style={{ width: `${Math.abs(vixCorrelation) * 100}%` }}
                    />
                </div>
            </div>

            {/* TICK Proxy */}
            <div className="text-center space-y-1">
                <p className="text-xs text-gray-500">TICK Proxy</p>
                <p className={`metric-value ${tickProxy > 0 ? 'text-green-400' :
                        tickProxy < 0 ? 'text-red-400' :
                            'text-gray-400'
                    }`}>
                    {formatMetric(tickProxy)}
                </p>
                <div className="h-1 w-full bg-gray-800 rounded-full overflow-hidden">
                    <div
                        className={`h-full transition-all ${tickProxy > 0 ? 'bg-green-500' : 'bg-red-500'
                            }`}
                        style={{ width: `${Math.abs(tickProxy) * 50}%` }}
                    />
                </div>
            </div>

            {/* Alpha Decay */}
            <div className="text-center space-y-1">
                <p className="text-xs text-gray-500">Alpha Decay</p>
                <p className={`metric-value ${alphaDecay < 0.3 ? 'text-green-400' :
                        alphaDecay < 0.7 ? 'text-yellow-400' :
                            'text-red-400'
                    }`}>
                    {formatMetric(alphaDecay)}
                </p>
                <div className="h-1 w-full bg-gray-800 rounded-full overflow-hidden">
                    <div
                        className="h-full bg-gradient-to-r from-green-500 to-red-500 transition-all"
                        style={{ width: `${alphaDecay * 100}%` }}
                    />
                </div>
            </div>
        </div>
    );
}
