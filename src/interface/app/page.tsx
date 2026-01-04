'use client';

import { AlphaRibbon } from '@/components/layout/AlphaRibbon';
import { SafetySidebar } from '@/components/layout/SafetySidebar';
import { DepthVault } from '@/components/zones/DepthVault';
import { RiemannWave } from '@/components/hud/RiemannWave';
import { ReasoningStream } from '@/components/debate/ReasoningStream';
import { ConsensusMeter } from '@/components/debate/ConsensusMeter';

export default function Home() {
  return (
    <main className="h-screen w-screen flex flex-col bg-[#0D1117] overflow-hidden text-white">
      {/* 1. Global Header (Alpha Ribbon) */}
      <AlphaRibbon />

      {/* 2. Main Workspace (Sidebar + 3-Column Grid) */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Rail: Safety Sidebar */}
        <SafetySidebar />

        {/* Triple Combat Grid */}
        <div className="flex-1 grid grid-cols-1 md:grid-cols-[300px_1fr_350px] gap-0.5 bg-white/5 p-0.5">

          {/* ZONE 1 (Left): Depth Vault */}
          <section className="bg-[#0D1117] rounded-sm overflow-hidden border border-white/5 flex flex-col">
            <DepthVault />
          </section>

          {/* ZONE 2 (Center): Kinetic HUD */}
          <section className="bg-[#0D1117] rounded-sm overflow-hidden border border-white/5 relative flex flex-col">
            {/* Viz Container */}
            <div className="flex-1 relative">
              {/* We use absolute positioning for the cloud to ensure it fills the flex container */}
              <div className="absolute inset-0">
                <RiemannWave />
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
              <span className="text-[10px] font-mono font-bold text-white/60 tracking-wider">REASONING STREAM</span>
            </div>
            <div className="flex-1 overflow-hidden relative">
              <ReasoningStream />
            </div>
          </section>

        </div>
      </div>
    </main>
  );
}
