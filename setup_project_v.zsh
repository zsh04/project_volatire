#!/bin/zsh

# ==============================================================================
# PROJECT V: GENESIS SCRIPT (v1.1 - Zsh Optimized)
# Codename: Cottage Prime
# Objective: Initialize the Sovereign Construct (Rust/Python Decoupled)
# ==============================================================================

# Exit immediately if a command exits with a non-zero status
set -e

PROJECT_ROOT="Volatire"

print -P "%F{green}🚀 Initiating Project V: Genesis Protocol (Zsh)...%f"

# --- 1. Directory Structure ---
print -P "%F{blue}📂 Constructing Topography...%f"

# Zsh brace expansion is robust
mkdir -p "$PROJECT_ROOT"/{docs/{public,internal},protos,infra,scripts,logs}
mkdir -p "$PROJECT_ROOT"/src/{reflex/src,brain/app}

cd "$PROJECT_ROOT"

# --- 2. The Master Documentation (The Law) ---
print -P "%F{yellow}📜 Inscribing The Constitution & Specifications...%f"

# 2.1 MASTER PLAN
cat <<'EOF' > docs/00_MASTER_INTEGRATION_PLAN.md
# 📜 Project V: Master Integration Plan (v2.0)
**Codename:** Cottage Prime
**Target:** Apple M4 Max (Bare Metal)
**Status:** ACTIVE

## 1. The Objective
Architect a sovereign, autonomous trading entity ("The Construct") that executes high-frequency decisions based on Physical State Vectors ($\Psi$), utilizing a **Decoupled Architecture** (Rust Reflex / Python Brain).

## 2. The Execution Phases

### Phase 1: The Foundation (Week 1)
* [ ] **Dir-01 (Genesis):** Initialize Repo, Docker (Dragonfly/QuestDB), and .proto definitions.
* [ ] **Dir-02 (The Mesh):** Build the gRPC Service Interface (brain.proto, reflex.proto).
* [ ] **Dir-03 (Infrastructure):** Deploy the Storage Trinity via Docker.

### Phase 2: The Reflex (Rust Standalone)
* [ ] **Dir-04 (The Body):** Build the Rust Binary.
* [ ] **Dir-05 (Feynman):** Implement Kinematics (v, j, H) in pure Rust.
* [ ] **Dir-06 (Simons):** Implement the ESN Reservoir (O(1)).
* [ ] **Dir-07 (Taleb):** Implement the "Ratchet Protocol" (Dynamic Risk).

### Phase 3: The Brain (Python Consciousness)
* [ ] **Dir-08 (The Mind):** Build the Python gRPC Client with pydantic-ai.
* [ ] **Dir-09 (Hypatia):** Connect DistilBERT to News Stream.
* [ ] **Dir-10 (Boyd):** Connect Gemma 2 to the Strategy Loop.
EOF

# 2.2 PHILOSOPHY (The Constitution v2.4)
cat <<'EOF' > docs/01_PHILOSOPHY.md
# 📜 The Project V Constitution (v2.4)
**Status:** REVIEW
**Codename:** The Sovereign Construct (Micro-Account Hardened)

## I. PREAMBLE: THE SOVEREIGNTY DOCTRINE
1. **Sovereignty:** We run on **Bare Metal**. We reject the Cloud.
2. **Physics over Finance:** We trade the State Vector ($\Psi$), not Price.
3. **Speed is Survival:** The system operates in **$O(1)$** complexity.
4. **Honesty:** We optimize for "Brutal Truth" over "False Precision."

## II. THE ONTOLOGY: QUANTUM SUPERPOSITION
The market is a Probability Density.
* **Action:** We **blend** strategies based on $P_{Riemann}$.
* **Collapse:** If $P_{Riemann} < 0.4$, collapse to **Cash**.

## III. THE THERMODYNAMICS OF VALUE (L1 ONLY)
* **Mass ($m$):** Executed Volume $\times$ CLV.
* **Efficiency ($\eta$):** Work (Price Change) / Energy (Volume).
* **Friction Law:** If $\eta$ is Low, we enter **Maker-Only Mode**.

## IV. THE AXIOM OF THE ORGANISM
* **Decoupling:** Reflex (Rust) handles Survival. Brain (Python) handles Strategy.
* **The Interlock:** If Brain heartbeat fails (>500ms), Reflex halts trading.
* **The Shiver:** Reflex autonomously ratchets size to minimum (0.01) if Efficiency drops (1-min loop).

## V. THE GOVERNANCE TRIAD
1. **Legislature (Offline):** Proposes parameter updates.
2. **Judiciary (CI/CD):** Vetoes updates failing the "Tunneling Test" (Gap-to-Zero).
3. **Executive (Real-time):** The Ratchet Protocol. Can unilaterally tighten risk rules in $O(1)$.
EOF

# 2.3 ARCHITECTURE
cat <<'EOF' > docs/02_ARCHITECTURE.md
# 🏗️ System Architecture (v2.3)
**Pattern:** Decoupled Microservices (Rust/Python via gRPC)

## 1. The Nervous System
* **ReflexD (Rust):** The Body. Market Data, Physics, ESN, Risk. (Port 50052)
* **BrainD (Python):** The Mind. LLM, Forecasting, Context. (Port 50051)
* **Gateway (Node):** UI Bridge. (Port 8080)
* **Protocol:** gRPC (Protobuf) over Unix Domain Sockets or Localhost.

## 2. The Data Flow
1. **Ingest:** ReflexD consumes WebSocket Ticks.
2. **React:** ReflexD calculates Physics & ESN Signal. Checks Risk. Executes.
3. **Inform:** ReflexD streams StateUpdate to BrainD.
4. **Think:** BrainD processes Context.
5. **Adjust:** BrainD sends StrategyUpdate (e.g., "Regime: Bear") to ReflexD.

## 3. Storage Trinity
* **Hot:** Dragonfly (Redis-compatible). Live State.
* **Warm:** LanceDB. Vector Embeddings.
* **Cold:** QuestDB. Tick History.
EOF

# 2.4 THE COUNCIL
cat <<'EOF' > docs/03_COUNCIL.md
# 🧠 The Council of Giants (v2.1)

## 1. The Reflex (Rust)
* **Feynman:** Kinematics ($v, j, H$).
* **Simons:** ESN (Echo State Network).
* **Taleb:** The Ratchet (Risk Gate).

## 2. The Brain (Python)
* **Kepler:** Chronos-Bolt Forecasting.
* **Boyd:** Pydantic AI Strategist (Gemma 2).
* **Hypatia:** DistilBERT Sentiment.
EOF

# 2.5 ENGINEERING
cat <<'EOF' > docs/04_ENGINEERING.md
# ⚙️ Engineering Standards (v2.0)

## 1. Tech Stack
* **Reflex:** Rust (Tokio, Tonic, Ndarray). NO PYTHON BINDINGS.
* **Brain:** Python 3.12 (Pydantic AI, Polars).
* **Comms:** Protobuf v3.

## 2. The O(1) Mandate
* Rust must use fixed-size Ring Buffers. No dynamic allocation in hot loops.

## 3. Pydantic AI
* All Python agents must return typed Pydantic models.
EOF

# 2.6 GOVERNANCE
cat <<'EOF' > docs/05_GOVERNANCE.md
# ⚖️ Governance & Workflow (v2.0)

## 1. Directives
* All work must be tracked via Directives (Jira/Markdown).
* **No Code Rule:** No coding until Directive is ACTIVE.

## 2. The Black Box Doctrine
* Public docs in `docs/public`.
* Alpha logic in `docs/internal` (GitIgnored).
EOF

# --- 3. The Nervous System (Protobufs) ---
print -P "%F{cyan}🧠 Defining The Synapse (Protobufs)...%f"

cat <<'EOF' > protos/brain.proto
syntax = "proto3";
package brain;

// The Strategy Service (Python)
service BrainService {
  // Reflex sends State, Brain returns Strategy
  rpc Pulse (StateVector) returns (StrategyIntent);
}

message StateVector {
  double timestamp = 1;
  double price = 2;
  double velocity = 3;
  double jerk = 4;
  double entropy = 5;
  double efficiency_index = 6; // Work / Energy
}

message StrategyIntent {
  string regime = 1; // "RIEMANN" or "SHANNON"
  double conviction = 2; // 0.0 to 1.0
  double risk_scalar = 3; // Multiplier for position sizing
  bool halt_trading = 4; // Kill switch
}
EOF

# --- 4. Infrastructure (Docker) ---
print -P "%F{magenta}🏗️ Laying Foundations (Docker)...%f"

cat <<'EOF' > infra/docker-compose.yml
services:
  dragonfly:
    image: 'docker.dragonflydb.io/dragonflydb/dragonfly'
    ulimits:
      memlock: -1
    ports:
      - "6379:6379"
    command: --default_lua_flags=allow-undeclared-keys
    volumes:
      - dragonfly_data:/data

  questdb:
    image: 'questdb/questdb'
    ports:
      - "9000:9000"
      - "9009:9009"
      - "8812:8812"
    volumes:
      - questdb_data:/var/lib/questdb

volumes:
  dragonfly_data:
  questdb_data:
EOF

# --- 5. Git Initialization ---
print -P "%F{white}📦 Initializing Repository...%f"

cat <<'EOF' > .gitignore
# Python
__pycache__/
*.py[cod]
.venv
.env

# Rust
target/
**/*.rs.bk

# Project V Specific
docs/internal/
*.log
data/
.DS_Store
EOF

# Initialize Git
git init -q
git add .
git commit -m "feat: Genesis - Project V Initial Commit [Directive-01]"

print -P "%F{green}✅ Project V: Genesis Complete.%f"
print -P "   -> Location: %B./$PROJECT_ROOT%b"
print -P "   -> Next Step: %Bcd $PROJECT_ROOT/infra && docker-compose up -d%b"