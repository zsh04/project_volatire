// D-84: Browser-side stress test utilities
// This module provides control functions for stress testing from the browser console

export interface StressTestControl {
    startSyntheticTicks: (rate?: number) => void;
    stopSyntheticTicks: () => void;
    getFPS: () => number;
    resetFPS: () => void;
}

// Get reference to telemetry worker
function getTelemetryWorker(): Worker | null {
    // This would need to be exposed from the app context
    // For now, return null and implement via window object
    return (window as any).__telemetry_worker__ || null;
}

export const StressTest: StressTestControl = {
    startSyntheticTicks: (rate = 5000) => {
        const worker = getTelemetryWorker();
        if (worker) {
            worker.postMessage({
                type: 'START_SYNTHETIC_TICKS',
                payload: { rate }
            });
            console.log(`[StressTest] Started synthetic ticks at ${rate}/sec`);
        } else {
            console.error('[StressTest] Telemetry worker not found');
        }
    },

    stopSyntheticTicks: () => {
        const worker = getTelemetryWorker();
        if (worker) {
            worker.postMessage({ type: 'STOP_SYNTHETIC_TICKS' });
            console.log('[StressTest] Stopped synthetic ticks');
        }
    },

    getFPS: () => {
        // Import from RiemannWave
        const { getCurrentFPS } = require('@/components/hud/RiemannWave');
        return getCurrentFPS();
    },

    resetFPS: () => {
        const { resetFPSCounter } = require('@/components/hud/RiemannWave');
        resetFPSCounter();
    }
};

// Expose to window for console access
if (typeof window !== 'undefined') {
    (window as any).StressTest = StressTest;
}
