import React, { useState } from 'react';
import { sendSovereignCommand, SovereignCommand, confirmCommand } from '@/lib/governance';
import { useSystemStore } from '@/lib/stores/system-store';
import { useNullification } from '../src/hooks/useNullification';

/**
 * Directive-86: Command Deck
 * High-priority command interface for pilot strategic oversight
 * Updated for D-90: System Sanity Score
 */
export function CommandDeck() {
  const [isPaused, setIsPaused] = useState(false);
  const [sentimentOverride, setSentimentOverride] = useState<number | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  // D-90: Sanity Score & D-88: Nullification
  const systemSanityScore = useSystemStore((state) => state.systemSanityScore);
  const { isNullified } = useNullification();

  const handleCommand = async (command: SovereignCommand, payload?: number) => {
    if (isProcessing) return;

    // Request confirmation for dangerous commands
    if (![SovereignCommand.RESUME, SovereignCommand.CLEAR_SENTIMENT_OVERRIDE].includes(command)) {
      if (!confirmCommand(command)) {
        return;
      }
    }

    setIsProcessing(true);
    try {
      const response = await sendSovereignCommand(command, payload);
      console.log(`✓ ${command} executed in ${response.latency_ms}ms`);
    } catch (error) {
      console.error(`✗ ${command} failed:`, error);
      alert(`Command failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handlePause = async () => {
    const cmd = isPaused ? SovereignCommand.RESUME : SovereignCommand.PAUSE;
    await handleCommand(cmd);
    setIsPaused(!isPaused);
  };

  const handleKill = async () => {
    await handleCommand(SovereignCommand.KILL);
  };

  const handleVeto = async () => {
    await handleCommand(SovereignCommand.VETO);
  };

  const handleCloseAll = async () => {
    await handleCommand(SovereignCommand.CLOSE_ALL);
  };

  const handleSentimentSlider = async (value: number) => {
    setSentimentOverride(value);
    await handleCommand(SovereignCommand.SET_SENTIMENT_OVERRIDE, value);
  };

  const handleClearSentiment = async () => {
    setSentimentOverride(null);
    await handleCommand(SovereignCommand.CLEAR_SENTIMENT_OVERRIDE);
  };

  return (
    <div className={`command-deck ${isNullified ? 'nullified' : ''}`}>
      <style jsx>{`
        .command-deck {
          position: fixed;
          bottom: 20px;
          right: 20px;
          background: rgba(0, 0, 0, 0.9);
          border: 2px solid #00ff41;
          border-radius: 8px;
          padding: 16px;
          min-width: 320px;
          box-shadow: 0 0 20px rgba(0, 255, 65, 0.3);
          font-family: 'JetBrains Mono', monospace;
          z-index: 1000;
          transition: border-color 0.3s;
        }

        .command-deck.nullified {
            border-color: #555;
            opacity: 0.7;
        }

        .deck-header {
          font-size: 12px;
          font-weight: bold;
          color: #00ff41;
          margin-bottom: 12px;
          display: flex;
          justify-content: space-between;
          text-transform: uppercase;
          letter-spacing: 1px;
          border-bottom: 1px solid rgba(0, 255, 65, 0.3);
          padding-bottom: 8px;
        }

        .sanity-score {
            font-size: 10px;
            padding: 2px 6px;
            border-radius: 4px;
            background: #111;
            color: ${systemSanityScore > 0.8 ? '#00ff41' : systemSanityScore > 0.5 ? '#ffaa00' : '#ff0055'};
            border: 1px solid ${systemSanityScore > 0.8 ? '#00ff41' : systemSanityScore > 0.5 ? '#ffaa00' : '#ff0055'};
        }

        .control-section {
          margin-bottom: 16px;
        }

        .control-label {
          font-size: 10px;
          color: #00ff41;
          margin-bottom: 4px;
          opacity: 0.8;
        }

        .btn {
          width: 100%;
          padding: 10px 16px;
          font-size: 13px;
          font-weight: bold;
          font-family: 'JetBrains Mono', monospace;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn-pause {
          background: ${isPaused ? '#00ff41' : '#1a1a1a'};
          color: ${isPaused ? '#000' : '#00ff41'};
          border: 2px solid #00ff41;
          margin-bottom: 8px;
        }

        .btn-pause:hover:not(:disabled) {
          background: #00ff41;
          color: #000;
          box-shadow: 0 0 10px rgba(0, 255, 65, 0.5);
        }

        .btn-critical {
          background: #1a1a1a;
          color: #ff0055;
          border: 2px solid #ff0055;
          margin-bottom: 4px;
        }

        .btn-critical:hover:not(:disabled) {
          background: #ff0055;
          color: #000;
          box-shadow: 0 0 10px rgba(255, 0, 85, 0.5);
        }

        .btn-veto {
          background: #1a1a1a;
          color: #ffaa00;
          border: 2px solid #ffaa00;
          margin-bottom: 8px;
        }

        .btn-veto:hover:not(:disabled) {
          background: #ffaa00;
          color: #000;
          box-shadow: 0 0 10px rgba(255, 170, 0, 0.5);
        }

        .sentiment-control {
          background: rgba(0, 255, 65, 0.05);
          padding: 12px;
          border-radius: 4px;
          border: 1px solid rgba(0, 255, 65, 0.2);
        }

        .sentiment-value
 {
          font-size: 14px;
          color: #00ff41;
          margin-bottom: 8px;
          text-align: center;
          font-weight: bold;
        }

        .slider {
          width: 100%;
          height: 4px;
          border-radius: 2px;
          background: rgba(0, 255, 65, 0.2);
          outline: none;
          -webkit-appearance: none;
        }

        .slider::-webkit-slider-thumb {
          -webkit-appearance: none;
          appearance: none;
          width: 16px;
          height: 16px;
          border-radius: 50%;
          background: #00ff41;
          cursor: pointer;
          box-shadow: 0 0 5px rgba(0, 255, 65, 0.5);
        }

        .slider::-moz-range-thumb {
          width: 16px;
          height: 16px;
          border-radius: 50%;
          background: #00ff41;
          cursor: pointer;
          border: none;
        }

        .btn-clear {
          margin-top: 8px;
          padding: 6px;
          font-size: 10px;
          background: transparent;
          color: #00ff41;
          border: 1px solid rgba(0, 255, 65, 0.3);
        }

        .btn-clear:hover:not(:disabled) {
          background: rgba(0, 255, 65, 0.1);
        }

        .status-indicator {
          margin-top: 12px;
          padding: 8px;
          border-radius: 4px;
          font-size: 10px;
          text-align: center;
          background: ${isPaused ? 'rgba(255, 170, 0, 0.1)' : 'rgba(0, 255, 65, 0.1)'};
          color: ${isPaused ? '#ffaa00' : '#00ff41'};
          border: 1px solid ${isPaused ? 'rgba(255, 170, 0, 0.3)' : 'rgba(0, 255, 65, 0.3)'};
        }
      `}</style>

      <div className="deck-header">
        <span>⚡ Command</span>
        <span className="sanity-score" title="Fidelity Multiplier (Risk Adjustment)">
          ξ {(systemSanityScore * 100).toFixed(0)}%
        </span>
      </div>

      {/* Tactical Pause */}
      <div className="control-section">
        <div className="control-label">Tactical State</div>
        <button
          className="btn btn-pause"
          onClick={handlePause}
          disabled={isProcessing}
        >
          {isPaused ? '▶️ Resume Trading' : '⏸️ Tactical Pause'}
        </button>
      </div>

      {/* Action Buttons */}
      <div className="control-section">
        <div className="control-label">Emergency Controls</div>
        <button
          className="btn btn-critical"
          onClick={handleKill}
          disabled={isProcessing}
        >
          🛑 Kill All
        </button>
        <button
          className="btn btn-critical"
          onClick={handleCloseAll}
          disabled={isProcessing}
        >
          📛 Close All Positions
        </button>
        <button
          className="btn btn-veto"
          onClick={handleVeto}
          disabled={isProcessing}
        >
          ⛔ Veto Next Trade
        </button>
      </div>

      {/* Sentiment Override */}
      <div className="control-section">
        <div className="control-label">Sentiment Override</div>
        <div className="sentiment-control">
          <div className="sentiment-value">
            {sentimentOverride !== null
              ? `Manual: ${sentimentOverride.toFixed(1)}`
              : 'Auto (Hypatia)'}
          </div>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={sentimentOverride ?? 0.5}
            onChange={(e) => handleSentimentSlider(parseFloat(e.target.value))}
            className="slider"
            disabled={isProcessing}
          />
          {sentimentOverride !== null && (
            <button
              className="btn btn-clear"
              onClick={handleClearSentiment}
              disabled={isProcessing}
            >
              Clear Override
            </button>
          )}
        </div>
      </div>

      {/* Status Indicator */}
      <div className="status-indicator">
        {isNullified ? '⚠️ AUDITOR RECALIBRATING (NULLIFIED)' : (isPaused ? '⏸️ TACTICAL PAUSE ACTIVE' : '▶️ AUTONOMOUS MODE')}
      </div>
    </div>
  );
}
