import { create } from 'zustand';

// ==============================================================================
// Types
// ==============================================================================

export interface PersonaHealth {
    name: 'Simons' | 'Hypatia';
    status: 'active' | 'degraded' | 'offline';
    confidence: number; // 0-1
    lastReason: string;
}

export interface AgentDecision {
    action: 'BUY' | 'SELL' | 'HOLD';
    reason: string;
    confidence: number;
    timestamp: number;
}

export interface ReasoningEntry {
    id: string;
    persona: 'Simons' | 'Hypatia';
    message: string;
    timestamp: number;
    type: 'info' | 'warning' | 'veto';
}

export interface AgentStore {
    // Persona health
    simons: PersonaHealth;
    hypatia: PersonaHealth;

    // Consensus (alignment between Simons & Hypatia)
    consensusScore: number; // 0-1, where 1 = perfect alignment

    // Current decision
    decision: AgentDecision | null;

    // Reasoning stream (last N entries)
    reasoningStream: ReasoningEntry[];
    maxStreamSize: number;

    // Veto state
    vetoActive: boolean;
    vetoReason: string | null;

    // Actions
    updatePersona: (persona: 'Simons' | 'Hypatia', health: Partial<PersonaHealth>) => void;
    updateConsensus: (score: number) => void;
    setDecision: (decision: AgentDecision) => void;
    addReasoning: (entry: Omit<ReasoningEntry, 'id'>) => void;
    activateVeto: (reason: string) => void;
    deactivateVeto: () => void;
}

// ==============================================================================
// Store
// ==============================================================================

export const useAgentStore = create<AgentStore>((set) => ({
    // Initial state
    simons: {
        name: 'Simons',
        status: 'offline',
        confidence: 0,
        lastReason: 'Initializing...',
    },

    hypatia: {
        name: 'Hypatia',
        status: 'offline',
        confidence: 0,
        lastReason: 'Initializing...',
    },

    consensusScore: 0,
    decision: null,
    reasoningStream: [],
    maxStreamSize: 100,
    vetoActive: false,
    vetoReason: null,

    // Actions
    updatePersona: (persona, health) =>
        set((state) => {
            const key = persona.toLowerCase() as 'simons' | 'hypatia';
            return {
                [key]: { ...state[key], ...health },
            };
        }),

    updateConsensus: (score) => set({ consensusScore: score }),

    setDecision: (decision) => set({ decision }),

    addReasoning: (entry) =>
        set((state) => ({
            reasoningStream: [
                { ...entry, id: `${entry.timestamp}-${entry.persona}` },
                ...state.reasoningStream,
            ].slice(0, state.maxStreamSize),
        })),

    activateVeto: (reason) => set({ vetoActive: true, vetoReason: reason }),

    deactivateVeto: () => set({ vetoActive: false, vetoReason: null }),
}));
