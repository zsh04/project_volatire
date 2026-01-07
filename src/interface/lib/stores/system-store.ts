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

export interface ReasoningEvent {
    id: string;
    author: 'SOROS' | 'TALEB' | 'GOVERNOR' | 'CHRONOS';
    content: string;
    timestamp: number;
    status: 'active' | 'nullified' | 'consensus';
    confidence: number;
}

export interface Position {
    symbol: string;
    netSize: number;
    avgEntryPrice: number;
    unrealizedPnl: number;
    entryTimestamp: number;
    currentPrice: number;
}

export interface Order {
    orderId: string;
    symbol: string;
    side: string;
    quantity: number;
    limitPrice: number;
    status: string;
    timestamp: number;
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

    // D-81: Hot-Swap
    hotswapActive: boolean;
    setHotswapActive: (active: boolean) => void;
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

    // D-105: Fiscal Control Deck (Finance Slice)
    finance: {
        unrealizedPnl: number;
        realizedPnl: number;
        equity: number;
        balance: number;
        btcPosition: number;
        equityHistory: number[];
        positions: Position[];
        orders: Order[];
    };
    // D-106: Time Machine
    replay: {
        isReplaying: boolean;
        replayTime: number;
        replaySpeed: number;
        startTime: number;
        endTime: number;
    };
    updateFinance: (updates: Partial<SystemStore['finance']>) => void;
    setReplay: (updates: Partial<SystemStore['replay']>) => void;

    // Venue Sentry (Directive-67)
    isHalted: boolean;
    setHalted: (halted: boolean) => void;

    // Forensic Scrub (Directive-78)
    scrubMode: boolean;
    scrubTimestamp: number | null; // The exact time being viewed
    forensicEvents: ForensicEvent[];
    setScrubMode: (active: boolean) => void;
    setScrubTimestamp: (ts: number | null) => void;
    addForensicEvent: (event: ForensicEvent) => void;

    // D-83: Ignition
    ignitionStatus: 'HIBERNATION' | 'HARDWARECHECK' | 'WARMINGUP' | 'PENNYTRADE' | 'AWAITINGGEMMA' | 'IGNITED';
    setIgnitionStatus: (status: SystemStore['ignitionStatus']) => void;

    // D-90: Governor Sanity
    systemSanityScore: number;
    setSystemSanityScore: (score: number) => void;

    // D-103: Live Reasoning Stream
    reasoningTrace: ReasoningEvent[];
    addReasoningEvent: (event: ReasoningEvent) => void;

    // D-107: Legislation
    legislation: {
        bias: 'NEUTRAL' | 'LONG_ONLY' | 'SHORT_ONLY';
        aggression: number;
        makerOnly: boolean;
        hibernation: boolean;
    };
    updateLegislation: (updates: Partial<SystemStore['legislation']>) => void;
}

export interface ForensicEvent {
    id: string;
    timestamp: number;
    type: 'VETO' | 'DRIFT' | 'PHASE_SHIFT' | 'MANUAL';
    label: string;
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
            status: 'offline',
            metricLabel: 'Uptime',
            metricValue: '---',
            latency: 0,
            lastHeartbeat: 0,
        },
        kepler: {
            id: 'kepler',
            name: 'KEPLER',
            type: 'service',
            status: 'offline',
            metricLabel: 'Universe',
            metricValue: 0,
            metricUnit: 'tickers',
            latency: 0,
            lastHeartbeat: 0,
        },
        feynman: {
            id: 'feynman',
            name: 'FEYNMAN',
            type: 'service',
            status: 'offline',
            metricLabel: 'Entropy',
            metricValue: 0,
            latency: 0,
            lastHeartbeat: 0,
        },
        boyd: {
            id: 'boyd',
            name: 'BOYD GATE',
            type: 'service',
            status: 'offline',
            metricLabel: 'Loop',
            metricValue: 0,
            metricUnit: 'Hz',
            latency: 0,
            lastHeartbeat: 0,
        },
        chronos: {
            id: 'chronos',
            name: 'CHRONOS-T5',
            type: 'model',
            status: 'offline',
            metricLabel: 'Pred Delta',
            metricValue: '---',
            latency: 0,
            lastHeartbeat: 0,
        },
        gemma: {
            id: 'gemma',
            name: 'GEMMA-9B',
            type: 'model',
            status: 'offline',
            metricLabel: 'Tokens/s',
            metricValue: 0,
            latency: 0,
            lastHeartbeat: 0,
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
        realizedPnl: 0,
        equity: 0,
        balance: 0,
        btcPosition: 0,
        equityHistory: Array(50).fill(0), // Init with zeros
        positions: [],
        orders: [],
    },
    // D-106 Initial State
    replay: {
        isReplaying: false,
        replayTime: 0,
        replaySpeed: 1.0,
        startTime: 0,
        endTime: 0,
    },
    updateFinance: (updates) => set((state) => {
        const newHistory = updates.equity
            ? [...state.finance.equityHistory.slice(1), updates.equity]
            : state.finance.equityHistory;

        return {
            finance: { ...state.finance, ...updates, equityHistory: newHistory }
        };
    }),

    // Forensic Implementation
    scrubMode: false,
    scrubTimestamp: null,
    forensicEvents: [],
    setScrubMode: (active) => set({ scrubMode: active }),
    setScrubTimestamp: (ts) => set({ scrubTimestamp: ts }),
    addForensicEvent: (event) => set((state) => ({
        forensicEvents: [...state.forensicEvents, event]
    })),

    // D-81: Hot-Swap
    hotswapActive: false,
    setHotswapActive: (active) => set({ hotswapActive: active }),

    // D-83: Ignition
    ignitionStatus: 'HIBERNATION',
    setIgnitionStatus: (status) => set({ ignitionStatus: status }),

    // D-90: Governor Sanity
    systemSanityScore: 1.0,
    setSystemSanityScore: (score) => set({ systemSanityScore: score }),

    // D-103: Live Reasoning Stream
    reasoningTrace: [],
    addReasoningEvent: (event) => set((state) => ({
        reasoningTrace: [event, ...state.reasoningTrace].slice(0, 50) // Keep last 50
    })),
    // D-106 Replay Actions
    setReplay: (updates) => set((state) => ({
        replay: { ...state.replay, ...updates }
    })),

    // D-107: Legislation
    legislation: {
        bias: 'NEUTRAL',
        aggression: 1.0,
        makerOnly: false,
        hibernation: false,
    },
    updateLegislation: (updates) => set((state) => ({
        legislation: { ...state.legislation, ...updates }
    })),
}));
