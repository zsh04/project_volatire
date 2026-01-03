#!/bin/zsh

# Directive-55: Project V Setup Script
# "The Foundation of the Frontier"

set -e

echo "🚀 Initializing Project V Environment..."

# 1. Directory Structure
echo "📂 Verifying Directory Structure..."
mkdir -p data/questdb
mkdir -p data/dragonfly
mkdir -p logs/reflex
mkdir -p logs/brain

# 2. Permissions
echo "🔒 Setting Permissions..."
chmod -R 755 scripts
chmod +x scripts/*.py 2>/dev/null || true
# Ensure data directories are writable by Docker (often UID 1000 or 1001)
# On Mac this is usually fine due to virtualization, but good practice.

# 3. Environment Check
if [ ! -f .env ]; then
    echo "⚠️ .env file missing! Creating from template..."
    if [ -f .env.example ]; then
        cp .env.example .env
        echo "✅ Created .env from example. PLEASE EDIT IT."
    else
        echo "❌ .env.example not found. Creating empty .env."
        touch .env
    fi
fi

# 4. Docker Check
echo "🐳 Checking Docker..."
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running!"
    exit 1
fi
echo "✅ Docker is active."

# 5. Network Check
echo "🕸️  Checking Network Mesh..."
if ! docker network inspect voltaire_mesh >/dev/null 2>&1; then
    echo "   Creating voltaire_mesh..."
    docker network create --driver bridge --opt com.docker.network.bridge.name=v_mesh0 voltaire_mesh
    echo "✅ Network Created."
else
    echo "✅ voltaire_mesh exists."
fi

echo "✨ Environment Ready. Launch with: docker-compose -f infra/docker-compose.prod.yml up -d"