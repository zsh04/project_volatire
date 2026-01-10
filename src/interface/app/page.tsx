'use client';

import { useRef } from 'react';
import { AuthGuard } from '@/components/AuthGuard'; // D-103
import { AlphaRibbon } from '@/components/layout/AlphaRibbon';
import { TacticalSidebar } from '@/components/hud/TacticalSidebar';
import { SafetySidebar } from '@/components/layout/SafetySidebar';
import { DepthVault } from '@/components/zones/DepthVault';
import { RiemannWave } from '@/components/hud/RiemannWave';
import { ReasoningStream } from '@/components/debate/ReasoningStream';
import { ConsensusMeter } from '@/components/debate/ConsensusMeter';
import { PositionBlotter } from '@/components/blotter/PositionBlotter'; // D-103
import { OrderQueue } from '@/components/blotter/OrderQueue'; // D-105
import { ForensicScrub } from '@/components/hud/ForensicScrub';
import Vitality from '@/components/hud/Vitality'; // Directive-80
import { HotswapStatus } from '@/components/hud/HotswapStatus'; // Directive-81
import IgnitionSwitch from '@/components/hud/IgnitionSwitch'; // D-83
import { CommandDeck } from '@/components/CommandDeck'; // D-UX
import { useSystemStore } from '@/lib/stores/system-store';

export default function Home() {
  const systemSanityScore = useSystemStore((state) => state.systemSanityScore);

  // Determine Halo Class
  const isReplaying = useSystemStore((state) => state.replay.isReplaying);

  // Determine Halo Class
  const haloClass = isReplaying ? 'fidelity-halo-replay border-amber-500/50 shadow-[0_0_50px_rgba(245,158,11,0.2)]' :
    systemSanityScore > 0.8 ? 'fidelity-halo-laminar' :
      systemSanityScore > 0.5 ? 'fidelity-halo-degraded' :
        'fidelity-halo-critical';

  return (
    <AuthGuard>
      <main className={`h-screen w-screen flex flex-col bg-[#0D1117] overflow-hidden text-white transition-all duration-500 ${haloClass}`}>
        {/* 1. Global Header (Alpha Ribbon) */}
        <AlphaRibbon />

        {/* 2. Main Workspace (Sidebar + 3-Column Grid) */}
        <div className="flex-1 flex overflow-hidden mb-24"> {/* Margin Bottom for Command Deck */}
          {/* Left Rail: Safety Sidebar */}
          <SafetySidebar />

          {/* Triple Combat Grid */}
          <div className="flex-1 grid grid-cols-1 md:grid-cols-[300px_1fr_350px] gap-0.5 bg-white/5 p-0.5">

            {/* ZONE 1 (Left): Fiscal Deck (Blotter + Orders) & Depth Vault */}
            <section className="bg-[#0D1117] rounded-sm overflow-hidden border border-white/5 flex flex-col relative group">
              {/* Fiscal Deck (Top 75%) */}
              <div className="flex-1 border-b border-white/5 flex flex-col">
                <div className="h-1/2 border-b border-white/5 overflow-hidden">
                  <PositionBlotter />
                </div>
                <div className="h-1/2 overflow-hidden">
                  <OrderQueue />
                </div>
              </div>
              {/* Depth Vault (Bottom 25%) */}
              <div className="h-1/4 border-t border-white/5">
                <DepthVault />
              </div>
            </section>

            {/* ZONE 2 (Center): Kinetic HUD */}
            <section className="bg-[#0D1117] rounded-sm overflow-hidden border border-white/5 relative flex flex-col">
              {/* Viz Container */}
              <div className="flex-1 relative">
                {/* We use absolute positioning for the cloud to ensure it fills the flex container */}
                <div className="absolute inset-0">
                  <RiemannWave />
                </div>

                {/* D-78: Forensic Scrub Overlay */}
                <ForensicScrub />

                {/* D-80: Vitality Sentinel Overlay (Top Right) */}
                <div className="absolute top-4 right-4 z-20 flex flex-col gap-2 items-end">
                  <Vitality />
                  <HotswapStatus />
                  <IgnitionSwitch />
                </div>
              </div>

              {/* Bottom: Consensus Meter (Floating above bottom edge) */}
              <div className="h-12 border-t border-white/5 bg-[#0D1117]/80 backdrop-blur px-4 flex items-center">
                <ConsensusMeter />
              </div>
            </section>

            {/* ZONE 3 (Right): Debate Feed */}
            <section className="bg-[#0D1117] rounded-sm overflow-hidden border border-white/5 flex flex-col">
              <div className="h-8 flex items-center px-3 bg-white/5 border-b border-white/5">
                <span className="text-[10px] font-mono font-bold text-white/60 tracking-wider">TRADE AUDIT JOURNAL</span>
              </div>
              <div className="flex-1 overflow-hidden relative">
                <ReasoningStream />
              </div>
            </section>

          </div>
        </div>

        {/* D-UX: Sovereign Command Deck Footer */}
        <CommandDeck />
      </main>
    </AuthGuard>
  );
}
