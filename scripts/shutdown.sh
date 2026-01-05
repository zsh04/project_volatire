#!/bin/bash
# scripts/shutdown.sh
# Gracefully shutdown all Voltaire services

echo "🛑 Initiating Voltaire System Shutdown..."

# Function to kill process by port
# Function to kill process by port gracefully
kill_port() {
    PORT=$1
    NAME=$2
    PID=$(lsof -t -i:$PORT)
    if [ -n "$PID" ]; then
        echo "   > Signalling $NAME (PID $PID) to stop..."
        kill $PID # Sends SIGTERM (15) by default
        
        # Wait up to 5 seconds for it to exit
        for i in {1..10}; do
            if ! ps -p $PID > /dev/null; then
                echo "     ✓ $NAME stopped gracefully."
                return
            fi
            sleep 0.5
        done
        
        echo "     ⚠️ $NAME did not exit. Forcing kill..."
        kill -9 $PID
    else
        echo "   > $NAME is already stopped."
    fi
}

# 0. Infrastructure (Docker)
echo "🐳 Stopping Docker Infrastructure..."
docker-compose -f infra/docker-compose.yml down
echo "   > Infrastructure stopped."

# 1. Kill Frontend (Next.js)
kill_port 3000 "Frontend"

# 2. Kill Envoy Proxy
kill_port 8080 "Envoy"
kill_port 9901 "Envoy Admin"

# 3. Kill Reflex Engine (Rust)
kill_port 50051 "Reflex"
# Fallback: Kill by name
pkill -f "target/release/reflex" && echo "   > Reflex process killed."

# 4. Kill Brain Service (Python)
kill_port 50052 "Brain"
# Fallback: Kill by name
pkill -f "src/server.py" && echo "   > Brain process killed."

# 5. Stop Ollama (Optional - comment out if you want to keep LLM loaded)
echo "🦙 Stopping Ollama..."
pkill -f "ollama serve"
echo "   > Ollama stopped."

echo "🧹 Cleaning up..."
# Optional: rm *.log

echo "✅ System Shutdown Complete."
