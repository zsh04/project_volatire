#!/bin/bash
# scripts/ignite.sh
# Ignition Sequence: Envoy -> Reflex -> Brain -> Frontend

set -e

echo "🚀 Initiating Voltaire Ignition Sequence..."

# 0. Pre-Flight Check: Model Health
echo "🔍 Checking AI Subsystems..."
cd src/brain && poetry run python check_models.py
if [ $? -eq 0 ]; then
    echo "   ✅ Models Operational."
else
    echo "   ❌ Model Check Failed. Aborting."
    exit 1
fi
cd ../../

# 1. Start Envoy
echo "🌐 Launching Envoy Proxy..."
# Assuming envoy is installed or running via docker. 
# Since user had it running on 8080, we'll try to use the infra config.
# If envoy is not found, we warn.
if command -v envoy &> /dev/null; then
    nohup envoy -c infra/envoy.yaml > envoy.log 2>&1 &
    echo "   > Envoy ignited (PID $!)."
else
    echo "   ⚠️ Envoy binary not found. Assuming it's running externally or via Docker."
fi

# 2. Start Reflex Engine (Rust)
echo "🦀 Launching Reflex Engine..."
cd src/reflex
nohup cargo run --release --bin reflex > ../../reflex.log 2>&1 &
echo "   > Reflex ignited (PID $!)."
cd ../../

# 3. Start Brain Service (Python)
echo "🧠 Launching Brain Service..."
cd src/brain
export PYTHONPATH=../../
nohup poetry run python src/server.py > ../../brain.log 2>&1 &
echo "   > Brain ignited (PID $!)."
cd ../../

# 4. Start Frontend (Next.js)
echo "⚛️ Launching Frontend..."
cd src/interface
nohup npm run dev > ../../frontend.log 2>&1 &
echo "   > Frontend ignited (PID $!)."
cd ../../

echo "✨ Ignition Sequence Complete. Monitoring logs..."
echo "📊 Run 'tail -f *.log' to monitor."
