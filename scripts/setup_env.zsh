#!/bin/zsh
# scripts/setup_env.zsh

set -e

print -P "%F{cyan}🚀 Project V: Environment Initialization (Python 3.12 Pinned)...%f"

# --- 1. Check Rust (Reflex) ---
if ! command -v cargo &> /dev/null; then
    print -P "%F{red}❌ Rust not found. Installing via rustup...%f"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
else
    print -P "%F{green}✅ Rust Detected:%f $(cargo --version)"
fi

# --- 2. Check uv (Brain Package Manager) ---
if ! command -v uv &> /dev/null; then
    print -P "%F{yellow}⚠️ 'uv' not found. Installing via Homebrew...%f"
    if command -v brew &> /dev/null; then
        brew install uv
    else
        curl -LsSf https://astral.sh/uv/install.sh | sh
    fi
else
    print -P "%F{green}✅ uv Detected:%f $(uv --version)"
fi

# --- 3. Initialize Brain Environment (STRICT 3.12) ---
print -P "%F{green}🧠 Materializing Brain Environment (Target: 3.12)...%f"

cd src/brain || exit

# Force uv to use Python 3.12. 
# It will fetch a standalone build if not found.
uv venv --python 3.12

# Activate and Install
source .venv/bin/activate

# Verify we actually got 3.12
CURRENT_PY=$(python --version)
if [[ "$CURRENT_PY" != *"3.12"* ]]; then
    print -P "%F{red}❌ CRITICAL: Failed to secure Python 3.12. Got: $CURRENT_PY%f"
    exit 1
fi
print -P "%F{green}✅ Locked on Target:%f $CURRENT_PY"

print -P "%F{blue}⬇️ Installing Dependencies...%f"
uv pip install .

cd ../..

# --- 4. Check Protoc ---
if ! command -v protoc &> /dev/null; then
    print -P "%F{yellow}⚠️ 'protoc' not found. Installing via Homebrew...%f"
    brew install protobuf
else
    print -P "%F{green}✅ Protoc Detected:%f $(protoc --version)"
fi

# --- 5. Check Docker ---
if ! docker info > /dev/null 2>&1; then
    print -P "%F{red}❌ Docker is NOT running. Start Docker Desktop!%f"
    exit 1
else
    print -P "%F{green}✅ Docker is Alive.%f"
fi

print -P "%F{green}🎉 Environment Ready (Python 3.12 Secured).%f"