import { create } from 'zustand';

export interface SystemNode {
    id: 'reflex' | 'kepler' | 'feynman' | 'boyd' | 'chronos' | 'gemma';
    name: string;
    type: 'kernel' | 'service' | 'model';
    status: 'online' | 'optimizing' | 'offline' | 'degraded';
    metricLabel: string;
    metricValue: string | number;
    metricUnit?: string;
    latency: number; // ms
    lastHeartbeat: number;
}

interface SystemStore {
    nodes: Record<string, SystemNode>;

    // Actions
    updateNode: (id: string, updates: Partial<SystemNode>) => void;
    setNodeStatus: (id: string, status: SystemNode['status']) => void;

    // Staircase Actions
    staircase: StaircaseState;
    updateStaircase: (updates: Partial<StaircaseState>) => void;

    // Regime Detector (Directive-65)
    currentRegime: 'LAMINAR' | 'TURBULENT' | 'DECOHERENT';
    setRegime: (regime: 'LAMINAR' | 'TURBULENT' | 'DECOHERENT') => void;

    // Audit Loop (Directive-66)
    audit: {
        driftScore: number; // 0.0 - 1.0
        isRecalibrating: boolean;
    };
    updateAudit: (updates: Partial<SystemStore['audit']>) => void;

    // Venue Sentry (Directive-67)
    venue: {
        status: 'ONLINE' | 'DEGRADED' | 'OFFLINE';
        rtt: number; // ms
        feedLatency: number; // ms (One-Way)
        isDesynced: boolean; // True if feedLatency > 500ms
        lastUpdate: number;
    };
    updateVenue: (updates: Partial<SystemStore['venue']>) => void;

    // Finance (Directive-72)
    finance: {
        unrealizedPnl: number;
        equity: number;
        balance: number;
    };
    updateFinance: (updates: Partial<SystemStore['finance']>) => void;

    // Kill Switch
    isHalted: boolean;
    setHalted: (halted: boolean) => void;
}

export interface StaircaseState {
    currentTier: 'Q0' | 'Q1' | 'Q2' | 'Q3' | 'Q4' | 'MAX';
    nextTierProgress: number; // 0 to 50
    isCooldown: boolean;
    cooldownRemaining: number;
    vetoCount: number;
}

export const useSystemStore = create<SystemStore>((set) => ({
    nodes: {
        reflex: {
            id: 'reflex',
            name: 'REFLEX KERNEL',
            type: 'kernel',
            status: 'online',
            metricLabel: 'Uptime',
            metricValue: '04:20:00',
            latency: 0,
            lastHeartbeat: Date.now(),
        },
        kepler: {
            id: 'kepler',
            name: 'KEPLER',
            type: 'service',
            status: 'online',
            metricLabel: 'Universe',
            metricValue: 10420,
            metricUnit: 'tickers',
            latency: 120,
            lastHeartbeat: Date.now(),
        },
        feynman: {
            id: 'feynman',
            name: 'FEYNMAN',
            type: 'service',
            status: 'optimizing',
            metricLabel: 'Entropy',
            metricValue: 0.84,
            latency: 45,
            lastHeartbeat: Date.now(),
        },
        boyd: {
            id: 'boyd',
            name: 'BOYD GATE',
            type: 'service',
            status: 'online',
            metricLabel: 'Loop',
            metricValue: 14,
            metricUnit: 'Hz',
            latency: 14,
            lastHeartbeat: Date.now(),
        },
        chronos: {
            id: 'chronos',
            name: 'CHRONOS-T5',
            type: 'model',
            status: 'online',
            metricLabel: 'Pred Delta',
            metricValue: '+0.4%',
            latency: 800,
            lastHeartbeat: Date.now(),
        },
        gemma: {
            id: 'gemma',
            name: 'GEMMA-9B',
            type: 'model',
            status: 'offline', // Default offline
            metricLabel: 'Tokens/s',
            metricValue: 0,
            latency: 0,
            lastHeartbeat: Date.now(),
        },
    },

    updateNode: (id, updates) => set((state) => ({
        nodes: {
            ...state.nodes,
            [id]: { ...state.nodes[id], ...updates, lastHeartbeat: Date.now() }
        }
    })),

    setNodeStatus: (id, status) => set((state) => ({
        nodes: {
            ...state.nodes,
            [id]: { ...state.nodes[id], status, lastHeartbeat: Date.now() }
        }
    })),

    // Staircase Implementation
    staircase: {
        currentTier: 'Q0',
        nextTierProgress: 0, // 0-50
        isCooldown: false,
        cooldownRemaining: 0,
        vetoCount: 0,
    },

    updateStaircase: (updates) => set((state) => ({
        staircase: { ...state.staircase, ...updates }
    })),

    // Regime Implementation
    currentRegime: 'LAMINAR',
    setRegime: (regime) => set({ currentRegime: regime }),

    // Audit Implementation
    audit: {
        driftScore: 0.0,
        isRecalibrating: false,
    },
    updateAudit: (updates) => set((state) => ({
        audit: { ...state.audit, ...updates }
    })),

    // Venue Implementation
    venue: {
        status: 'ONLINE',
        rtt: 45, // ms
        feedLatency: 0,
        isDesynced: false,
        lastUpdate: Date.now(),
    },
    updateVenue: (updates) => set((state) => ({
        venue: { ...state.venue, ...updates }
    })),

    // Kill Switch (Directive-68)
    isHalted: false,
    setHalted: (halted) => set({ isHalted: halted }),

    // Finance (Directive-72: Live Data Integrity)
    finance: {
        unrealizedPnl: 0,
        equity: 0,
        balance: 0,
    },
    updateFinance: (updates) => set((state) => ({
        finance: { ...state.finance, ...updates }
    })),
}));
