#!/bin/zsh
set -e

# ==========================================
# Directive-05: The First Pulse Orchestrator
# ==========================================

echo "⚡ [GENESIS] Initiating System Pulse Verification..."

# 0. Setup Cleanup Trap (Kill background processes on exit)
trap 'kill $(jobs -p) 2>/dev/null || true' EXIT

# 1. Clean & Build Reflex (The Body)
echo "🦀 [REFLEX] Building Release Binary..."
cd src/reflex
cargo build --release
REFLEX_BIN="src/reflex/target/release/reflex"
cd ../..

# 2. Check Virtual Env (The Mind)
if [ ! -d ".venv" ]; then
    echo "❌ [BRAIN] No virtual environment found. Run setup first."
    exit 1
fi
PYTHON=".venv/bin/python"
PYTEST=".venv/bin/pytest"

# 3. Start The Brain (Mind)
echo "🧠 [BRAIN] Waking up..."
export PYTHONPATH=$PYTHONPATH:$(pwd)
$PYTHON -m src.brain.app.main &
BRAIN_PID=$!
echo "🧠 [BRAIN] PID: $BRAIN_PID"

# 4. Start The Reflex (Body)
echo "💪 [REFLEX] Waking up..."
$REFLEX_BIN &
REFLEX_PID=$!
echo "💪 [REFLEX] PID: $REFLEX_PID"

# 5. Wait for Vital Signs (Ports 50051 & 50052)
echo "⏳ [WAIT] Stabilizing Bi-Cameral Connection..."
sleep 5

# Optional: Check ports explicitly if nc is available
if command -v nc >/dev/null 2>&1; then
    if ! nc -z localhost 50051; then echo "❌ Reflex Port 50051 Closed"; exit 1; fi
    if ! nc -z localhost 50052; then echo "❌ Brain Port 50052 Closed"; exit 1; fi
fi

# 6. Apply Stimulus (Run Tests)
echo "🔬 [TEST] Applying Stimulus..."
export PYTHONPATH=$PYTHONPATH:$(pwd)
$PYTEST tests/integration/test_pulse.py -v

echo "✅ [SUCCESS] System Pulse Verified. Architecture is Alive."
