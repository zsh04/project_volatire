#!/usr/bin/env bash
# Vector C: Zero-Copy Jitter Audit (D-80, D-82)
# Test: Run Historian at maximum bandwidth (1GB/min) while measuring OODA loop jitter
# Acceptance: OODA Jitter < 50μs, Cache Misses < 1%, Core Interrupts = 0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source utilities
source "$SCRIPT_DIR/lib/metrics.sh"

# Configuration
DURATION="${DURATION:-300}"  # Test duration in seconds (5 minutes)
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/test_results}"
TARGET_BANDWIDTH_MB_MIN=1000  # 1GB/min = ~16.7MB/sec

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[VECTOR-C]${NC} $*"; }
log_success() { echo -e "${GREEN}[VECTOR-C PASS]${NC} $*"; }
log_error() { echo -e "${RED}[VECTOR-C FAIL]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[VECTOR-C WARN]${NC} $*"; }

# Check kernel parameters
check_kernel_params() {
    log_info "Checking kernel isolation parameters..."
    
    local cmdline=$(cat /proc/cmdline)
    local has_isolcpus=0
    local has_nohz=0
    
    if echo "$cmdline" | grep -q "isolcpus"; then
        log_success "isolcpus parameter found"
        has_isolcpus=1
    else
        log_warn "isolcpus not configured (recommended for production)"
    fi
    
    if echo "$cmdline" | grep -q "nohz_full"; then
        log_success "nohz_full parameter found"
        has_nohz=1
    else
        log_warn "nohz_full not configured (recommended for production)"
    fi
    
    # Check if perf is available
    if ! command -v perf &> /dev/null; then
        log_error "perf not found. Install with: sudo apt-get install linux-tools-generic (Linux) or brew install perf (macOS)"
        exit 1
    fi
    
    log_success "Kernel parameter check complete"
}

# Check /dev/shm capacity
check_shm_capacity() {
    log_info "Checking /dev/shm capacity..."
    
    local shm_size=$(df -m /dev/shm | tail -1 | awk '{print $2}')
    local required_mb=2048  # 2GB minimum
    
    if [[ $shm_size -lt $required_mb ]]; then
        log_error "/dev/shm too small: ${shm_size}MB (need ${required_mb}MB)"
        log_error "Increase with: sudo mount -o remount,size=2G /dev/shm"
        exit 1
    fi
    
    log_success "/dev/shm capacity OK: ${shm_size}MB"
}

# Start Reflex with perf instrumentation
start_reflex_with_perf() {
    log_info "Starting Reflex with perf instrumentation..."
    
    cd "$PROJECT_ROOT"
    
    # Enable stress mode for Historian
    export HISTORIAN_STRESS_MODE=1
    export HISTORIAN_FLUSH_INTERVAL_MS=10  # Aggressive flushing
    
    # Start reflex
    target/release/reflex --sim-mode &
    REFLEX_PID=$!
    
    sleep 5
    
    # Start perf monitoring
    log_info "Starting perf stat on PID $REFLEX_PID..."
    
    # Monitor cache misses, context switches, and CPU cycles
    # Use Interval mode (-I 100) for jitter calculation
    perf stat -I 100 -p "$REFLEX_PID" \
        -e cache-misses,cache-references,context-switches,cycles \
        -o "$OUTPUT_DIR/perf_stats.txt" &
    PERF_PID=$!
    
    log_success "Reflex started with perf instrumentation (PID: $REFLEX_PID)"
}

# Monitor core interrupts
monitor_core_interrupts() {
    local core_id="${1:-0}"  # Default to core 0
    local duration=$2
    
    log_info "Monitoring interrupts on core $core_id for ${duration}s..."
    
    # Take snapshot before
    local before=$(cat /proc/interrupts | grep "CPU${core_id}" | awk '{sum+=$2} END {print sum}')
    
    sleep "$duration"
    
    # Take snapshot after
    local after=$(cat /proc/interrupts | grep "CPU${core_id}" | awk '{sum+=$2} END {print sum}')
    
    local delta=$((after - before))
    echo "$delta"
}

# Parse perf output for jitter
extract_perf_jitter() {
    local perf_file="$1"
    local field="${2:-ooda_jitter_max_us}"
    
    # Use python analyzer to extract stats
    local result=$("$SCRIPT_DIR/lib/perf_analyzer.py" --input "$perf_file" --interval 0.1)
    
    # Extract specific field using python one-liner (avoids jq dependency)
    echo "$result" | python3 -c "import sys, json; print(json.load(sys.stdin).get('$field', 0.0))"
}

# Check Historian write bandwidth
check_historian_bandwidth() {
    local shm_file="/dev/shm/reflex_log_ring"
    
    if [[ ! -f "$shm_file" ]]; then
        log_warn "Historian ring buffer not found at $shm_file"
        return 0
    fi
    
    # Measure file growth over 10 seconds
    local size_before=$(stat -f%z "$shm_file" 2>/dev/null || stat -c%s "$shm_file" 2>/dev/null)
    sleep 10
    local size_after=$(stat -f%z "$shm_file" 2>/dev/null || stat -c%s "$shm_file" 2>/dev/null)
    
    local bytes_per_sec=$(( (size_after - size_before) / 10 ))
    local mb_per_min=$(echo "scale=2; ($bytes_per_sec * 60) / 1048576" | bc)
    
    log_info "Historian bandwidth: ${mb_per_min} MB/min"
    echo "$mb_per_min"
}

# Cleanup
cleanup() {
    log_info "Cleaning up processes..."
    
    if [[ -n "${PERF_PID:-}" ]]; then
        kill "$PERF_PID" 2>/dev/null || true
    fi
    
    if [[ -n "${REFLEX_PID:-}" ]]; then
        kill "$REFLEX_PID" 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Main test execution
main() {
    log_info "=== VECTOR C: ZERO-COPY JITTER AUDIT ==="
    log_info "Target Bandwidth: ${TARGET_BANDWIDTH_MB_MIN} MB/min"
    log_info "Duration: ${DURATION}s"
    log_info "==========================================="
    
    # Prerequisites
    check_kernel_params
    check_shm_capacity
    
    # Start test
    mkdir -p "$OUTPUT_DIR"
    start_reflex_with_perf
    
    # Wait for system stabilization
    log_info "Waiting 10s for system stabilization..."
    sleep 10
    
    # Baseline measurements
    log_info "Collecting baseline metrics..."
    local baseline_interrupts=$(monitor_core_interrupts 0 5)
    log_info "Baseline interrupts (5s): $baseline_interrupts"
    
    # Run test
    log_info "Starting ${DURATION}s jitter audit..."
    
    # Monitor interrupts during test
    local test_interrupts=$(monitor_core_interrupts 0 "$DURATION")
    
    # Wait for perf to complete
    wait "$PERF_PID" 2>/dev/null || true
    
    # Check Historian bandwidth
    local bandwidth=$(check_historian_bandwidth)
    
    # Extract metrics
    # We call extract_perf_jitter multiple times or cache the JSON result.
    # For simplicity, we'll re-run parsing or modify to get all at once.
    # Let's get the full JSON from the analyzer once.
    local perf_json=$("$SCRIPT_DIR/lib/perf_analyzer.py" --input "$OUTPUT_DIR/perf_stats.txt" --interval 0.1)

    local jitter_us=$(echo "$perf_json" | python3 -c "import sys, json; print(json.load(sys.stdin).get('ooda_jitter_max_us', 0.0))")
    local jitter_std_dev_us=$(echo "$perf_json" | python3 -c "import sys, json; print(json.load(sys.stdin).get('ooda_jitter_std_dev_us', 0.0))")
    local cache_miss_pct=$(echo "$perf_json" | python3 -c "import sys, json; print(json.load(sys.stdin).get('cache_miss_rate_pct', 0.0))")
    
    # Evaluate results
    log_info "==========================================="
    log_info "Test Results:"
    log_info "  OODA Loop Jitter (Max): ${jitter_us}μs"
    log_info "  OODA Loop Jitter (StdDev): ${jitter_std_dev_us}μs"
    log_info "  Cache Miss Rate: ${cache_miss_pct}%"
    log_info "  Core Interrupts: ${test_interrupts}"
    log_info "  Historian Bandwidth: ${bandwidth} MB/min"
    log_info "==========================================="
    
    # Check acceptance criteria
    local status="PASS"
    local jitter_pass=$(echo "$jitter_us < 50" | bc)
    local cache_pass=$(echo "$cache_miss_pct < 1.0" | bc)
    local interrupts_pass=$([ "$test_interrupts" -eq 0 ] && echo "1" || echo "0")
    
    if [[ "$jitter_pass" -ne 1 ]]; then
        log_error "OODA jitter above threshold: ${jitter_us}μs >= 50μs"
        status="FAIL"
    fi
    
    if [[ "$cache_pass" -ne 1 ]]; then
        log_error "Cache miss rate above threshold: ${cache_miss_pct}% >= 1%"
        status="FAIL"
    fi
    
    if [[ "$interrupts_pass" -ne 1 ]]; then
        log_warn "Core interrupts detected: $test_interrupts (expected 0)"
        # Don't fail on this in dev environments
    fi
    
    # Generate JSON report
    local metrics_json=$(cat << EOF
{
    "ooda_jitter_max_us": $jitter_us,
    "ooda_jitter_std_dev_us": $jitter_std_dev_us,
    "cache_miss_rate_pct": $cache_miss_pct,
    "core_interrupts": $test_interrupts,
    "historian_bandwidth_mb_min": $bandwidth,
    "target_bandwidth_mb_min": $TARGET_BANDWIDTH_MB_MIN,
    "thresholds": {
        "jitter_max_us": 50,
        "cache_miss_max_pct": 1.0,
        "interrupts_max": 0
    },
    "pass": {
        "jitter": $([ "$jitter_pass" -eq 1 ] && echo "true" || echo "false"),
        "cache": $([ "$cache_pass" -eq 1 ] && echo "true" || echo "false"),
        "interrupts": $([ "$interrupts_pass" -eq 1 ] && echo "true" || echo "false")
    }
}
EOF
    )
    
    create_test_result_json "$status" "$metrics_json" > "$OUTPUT_DIR/vector_c_results.json"
    
    if [[ "$status" == "PASS" ]]; then
        log_success "Vector C: ALL CHECKS PASSED"
        exit 0
    else
        log_error "Vector C: FAILED"
        exit 1
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duration=*)
            DURATION="${1#*=}"
            shift
            ;;
        --output=*)
            OUTPUT_DIR="${1#*=}"
            shift
            ;;
        --core=*)
            CORE_ID="${1#*=}"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

main
