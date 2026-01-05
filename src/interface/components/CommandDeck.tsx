import React, { useState, useRef } from 'react';
import { motion, useMotionValue, useTransform, useDragControls } from 'framer-motion';
import { sendSovereignCommand, SovereignCommand, confirmCommand } from '@/lib/governance';
import { useSystemStore } from '@/lib/stores/system-store';
import { useNullification } from '../src/hooks/useNullification';
import { AlertOctagon, PauseCircle, PlayCircle, ShieldBan, XCircle, Activity } from 'lucide-react';

/**
 * Directive-UX: Sovereign Command Deck
 * 
 * Re-Architected for "Action Oriented" UX.
 * Features:
 * - Swipe-to-Kill (prevents accidental trigger)
 * - Vitality Halo Feedback
 * - Compact "Glass Cockpit" aesthetic
 */

export function CommandDeck() {
  const [isPaused, setIsPaused] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  // D-90 & D-88
  const systemSanityScore = useSystemStore((state) => state.systemSanityScore);
  const { isNullified } = useNullification();

  // Swipe Logic
  const dragControls = useDragControls();
  const x = useMotionValue(0);
  const widthRef = useRef<HTMLDivElement>(null);

  // Slide Transforms
  const background = useTransform(
    x,
    [-200, 0, 200],
    ['rgba(255, 170, 0, 0.2)', 'rgba(0, 0, 0, 0.8)', 'rgba(255, 0, 85, 0.2)']
  );
  const borderColor = useTransform(
    x,
    [-200, 0, 200],
    ['#ffaa00', '#333', '#ff0055']
  );

  const handleCommand = async (command: SovereignCommand, payload?: number) => {
    if (isProcessing) return;
    setIsProcessing(true);
    try {
      if (command === SovereignCommand.KILL) {
        console.warn("💀 KILL SWITCH ACTIVATED VIA SWIPE");
      }
      const response = await sendSovereignCommand(command, payload);
      console.log(`✓ ${command} executed in ${response.latency_ms}ms`);
    } catch (error) {
      console.error(`✗ ${command} failed:`, error);
    } finally {
      setIsProcessing(false);
    }
  };

  const onDragEnd = async (_: any, info: any) => {
    const offset = info.offset.x;
    if (offset > 150) {
      // Swipe Right -> KILL
      if (confirmCommand(SovereignCommand.KILL)) {
        await handleCommand(SovereignCommand.KILL);
      }
    } else if (offset < -150) {
      // Swipe Left -> VETO
      await handleCommand(SovereignCommand.VETO);
    }
  };

  const handlePause = async () => {
    const cmd = isPaused ? SovereignCommand.RESUME : SovereignCommand.PAUSE;
    await handleCommand(cmd);
    setIsPaused(!isPaused);
  };

  return (
    <motion.div
      className={`fixed bottom-0 left-0 right-0 h-24 backdrop-blur-md border-t border-white/10 z-50 flex items-center justify-between px-8 transition-colors duration-300 ${isNullified ? 'grayscale opacity-50' : ''}`}
      style={{ background, borderTopColor: borderColor }}
    >
      {/* LEFT: Tactical Controls */}
      <div className="flex items-center gap-4">
        <button
          onClick={handlePause}
          disabled={isProcessing}
          className={`flex items-center gap-2 px-4 py-2 rounded-full border border-white/20 hover:border-white/50 transition-all ${isPaused ? 'bg-amber-500/20 text-amber-400' : 'bg-emerald-500/10 text-emerald-400'}`}
        >
          {isPaused ? <PlayCircle size={18} /> : <PauseCircle size={18} />}
          <span className="text-xs font-mono font-bold tracking-widest uppercase">
            {isPaused ? 'RESUME' : 'PAUSE'}
          </span>
        </button>

        <div className="h-8 w-px bg-white/10 mx-2" />

        <div className="flex flex-col">
          <span className="text-[10px] text-white/40 font-mono tracking-wider uppercase">System Sanity</span>
          <div className="flex items-center gap-2">
            <Activity size={14} className={systemSanityScore > 0.8 ? 'text-emerald-400' : 'text-red-500'} />
            <span className={`text-sm font-mono font-bold ${systemSanityScore > 0.8 ? 'text-emerald-400' : 'text-red-500'}`}>
              {(systemSanityScore * 100).toFixed(0)}%
            </span>
          </div>
        </div>
      </div>

      {/* CENTER: Swipe-to-Kill Slider */}
      <div className="relative w-[400px] h-12 bg-black/40 rounded-full border border-white/10 overflow-hidden flex items-center justify-center" ref={widthRef}>
        <div className="absolute inset-0 flex items-center justify-between px-6 pointer-events-none opacity-40">
          <span className="text-[10px] font-mono text-amber-500 tracking-widest">{'< VETO'}</span>
          <span className="text-[10px] font-mono text-red-500 tracking-widest">{'KILL >'}</span>
        </div>

        <motion.div
          drag="x"
          dragConstraints={{ left: -180, right: 180 }} // Constrained within pill
          dragElastic={0.1}
          dragControls={dragControls}
          onDragEnd={onDragEnd}
          style={{ x }}
          className="w-16 h-10 bg-white/10 backdrop-blur-lg rounded-full border border-white/30 shadow-[0_0_15px_rgba(255,255,255,0.1)] cursor-grab active:cursor-grabbing flex items-center justify-center relative z-10"
        >
          <div className="w-1 h-4 bg-white/50 rounded-full" />
          <div className="w-1 h-4 bg-white/50 rounded-full mx-1" />
          <div className="w-1 h-4 bg-white/50 rounded-full" />
        </motion.div>
      </div>

      {/* RIGHT: Status / Override */}
      <div className="flex items-center gap-4">
        {/* Close All (Immediate) - Kept as button for rapid liquidation */}
        <button
          onClick={() => handleCommand(SovereignCommand.CLOSE_ALL)}
          disabled={isProcessing}
          className="flex items-center gap-2 px-4 py-2 rounded-full border border-red-500/30 bg-red-500/5 hover:bg-red-500/20 text-red-400 transition-all"
        >
          <XCircle size={16} />
          <span className="text-xs font-mono font-bold tracking-wider">FLATTEN</span>
        </button>
      </div>
    </motion.div>
  );
}

