#!/bin/bash
# scripts/shutdown.sh
# Gracefully shutdown all Voltaire services

echo "🛑 Initiating Voltaire System Shutdown..."

# Function to kill process by port
kill_port() {
    PORT=$1
    NAME=$2
    PID=$(lsof -t -i:$PORT)
    if [ -n "$PID" ]; then
        echo "   > Killing $NAME (PID $PID) on port $PORT..."
        kill -9 $PID
    else
        echo "   > $NAME is already stopped."
    fi
}

# 1. Kill Frontend (Next.js)
kill_port 3000 "Frontend"

# 2. Kill Envoy Proxy
kill_port 8080 "Envoy"

# 3. Kill Reflex Engine (Rust)
kill_port 50051 "Reflex"

# 4. Kill Brain Service (Python)
kill_port 50052 "Brain"

echo "✅ System Shutdown Complete."
