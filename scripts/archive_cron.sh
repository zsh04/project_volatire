# 1. Navigate to Project Root
# Script is in voltaire/scripts, so project root is one level up
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT/src/reflex" || exit 1

# 2. Source Environment Variables
# Relying on cargo/dotenvy to load .env from project root
# (dotenvy searches parent directories)

# 3. Log Start
LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"

echo "----------------------------------------------------------------"
echo "Reflex Archiver Job Started at $(date)"
echo "Execution Dir: $(pwd)"
echo "Log Dir: $LOG_DIR"

# 4. Run Archiver
# Using cargo run relative to current dir (src/reflex)
cargo run --bin reflex -- --mode archive >> "$LOG_DIR/archiver_$(date +%Y-%m-%d).log" 2>&1

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "Reflex Archiver Job Completed Successfully at $(date)"
else
    echo "Reflex Archiver Job Failed at $(date) with Exit Code $EXIT_CODE"
fi

echo "----------------------------------------------------------------"
exit $EXIT_CODE
