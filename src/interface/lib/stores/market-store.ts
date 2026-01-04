import { create } from 'zustand';

// ==============================================================================
// Types
// ==============================================================================

export interface PhysicsState {
    price: number;
    velocity: number;
    acceleration: number;
    jerk: number;
    entropy: number;
    efficiencyIndex: number;
    timestamp: number;
}

export type RiemannState = 'momentum' | 'meanReversion' | 'transitioning';

export interface MarketStore {
    // Current physics
    physics: PhysicsState;

    // Riemann probability (momentum vs mean-reversion)
    riemannState: RiemannState;
    riemannProbability: number; // 0-1, where 1 = full momentum

    // Market internals
    vixCorrelation: number;
    tickProxy: number;
    alphaDecay: number;

    // Actions
    updatePhysics: (physics: Partial<PhysicsState>) => void;
    updateRiemann: (state: RiemannState, probability: number) => void;
    updateInternals: (vix: number, tick: number, alpha: number) => void;
}

// ==============================================================================
// Store
// ==============================================================================

export const useMarketStore = create<MarketStore>((set) => ({
    // Initial state
    physics: {
        price: 0,
        velocity: 0,
        acceleration: 0,
        jerk: 0,
        entropy: 0,
        efficiencyIndex: 0,
        timestamp: 0,
    },

    riemannState: 'transitioning',
    riemannProbability: 0.5,

    vixCorrelation: 0,
    tickProxy: 0,
    alphaDecay: 0,

    // Actions
    updatePhysics: (newPhysics) =>
        set((state) => ({
            physics: { ...state.physics, ...newPhysics },
        })),

    updateRiemann: (state, probability) =>
        set({ riemannState: state, riemannProbability: probability }),

    updateInternals: (vix, tick, alpha) =>
        set({ vixCorrelation: vix, tickProxy: tick, alphaDecay: alpha }),
}));
