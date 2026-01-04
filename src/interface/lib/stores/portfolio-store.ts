import { create } from 'zustand';

interface PortfolioStore {
    // Current System Tier (Data from Backend/Rust)
    systemRiskTier: number; // 0-5 (Q0-MAX)

    // Manual Risk Cap (Pilot Override)
    manualRiskCap: number | null; // 0-5 or null if uncapped
    setRiskCap: (level: number | null) => void;

    // Portfolio Metrics
    cash: number;
    positions: Record<string, number>;
}

export const usePortfolioStore = create<PortfolioStore>((set) => ({
    systemRiskTier: 2, // Default start at Q2 (0.10)
    manualRiskCap: null, // Default uncapped
    setRiskCap: (level) => set({ manualRiskCap: level }),

    cash: 100000,
    positions: {},
}));
