# Tutorial: Getting Started with Voltaire

**Target Audience:** New Developers / Quants
**Goal:** Run the system in Simulation Mode.

## Prerequisites

* macOS (Apple Silicon)
* Python 3.12+ (Strict)
* Rust (Latest Stable)
* Docker Desktop

## Step 1: Environment Setup

```bash
# Clone the repo
git clone https://github.com/project-voltaire/voltaire.git
cd voltaire

# Brain Setup (Python)
cd src/brain
python3.12 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Reflex Setup (Rust)
cd ../reflex
cargo build --release
```

## Step 2: Running a Simulation

1. Start the Reflex Server:

    ```bash
    ./target/release/reflex_server --mode SIMULATION
    ```

2. Start the Brain Service:

    ```bash
    export VOLTAIRE_ENV=SIMULATION
    python src/server.py
    ```

3. Observe the logs for the "Heartbeat" handshake.

## Next Steps

Check out the [How-To Guides](../how-to/) for backfilling data.
