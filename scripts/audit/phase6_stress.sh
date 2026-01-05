#!/usr/bin/env bash
# Directive-84: Phase 6 Forensic Audit (The Stress Gate)
# Main orchestrator for the "Chaos Monkey" stress testing protocol

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Source utilities
source "$SCRIPT_DIR/lib/metrics.sh"

# Configuration
DURATION="${DURATION:-300}"  # Default 5 minutes
VECTOR="${VECTOR:-all}"      # all, telemetry, brain, jitter
REPORT_FORMAT="${REPORT_FORMAT:-json}"  # json, text
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/test_results}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

# Print banner
print_banner() {
    cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   DIRECTIVE-84: PHASE 6 FORENSIC AUDIT (STRESS GATE)    ║
║                                                           ║
║   "The Chaos Monkey Protocol"                           ║
║   Testing: OODA Loop | Riemann Wave | Gemma Reasoning   ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing=0
    
    # Check for required tools
    for tool in jq bc perf; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool not found: $tool"
            ((missing++))
        fi
    done
    
    # Check if reflex binary exists
    if [[ ! -f "$PROJECT_ROOT/target/release/reflex" ]]; then
        log_warn "Reflex binary not found in release mode. Building..."
        cd "$PROJECT_ROOT/src/reflex"
        cargo build --release
    fi
    
    # Check kernel parameters (for Vector C)
    if [[ "$VECTOR" == "all" ]] || [[ "$VECTOR" == "jitter" ]]; then
        if ! grep -q "isolcpus" /proc/cmdline; then
            log_warn "Kernel parameter 'isolcpus' not set. Vector C may fail."
        fi
    fi
    
    if [[ $missing -gt 0 ]]; then
        log_error "Missing $missing required tools. Aborting."
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Initialize test environment
init_test_env() {
    log_info "Initializing test environment..."
    
    mkdir -p "$OUTPUT_DIR"
    
    # Create test report structure
    cat > "$OUTPUT_DIR/test_manifest.json" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "test_id": "$(uuidgen)",
    "duration_sec": $DURATION,
    "vectors_requested": "$VECTOR",
    "status": "RUNNING"
}
EOF
    
    log_success "Test environment initialized"
}

# Run Vector A: Telemetry Burst
run_vector_a() {
    log_info "Starting Vector A: Telemetry Burst Test..."
    
    if [[ -f "$SCRIPT_DIR/vector_a_telemetry_burst.sh" ]]; then
        bash "$SCRIPT_DIR/vector_a_telemetry_burst.sh" --duration="$DURATION" --output="$OUTPUT_DIR"
        local exit_code=$?
        
        if [[ $exit_code -eq 0 ]]; then
            log_success "Vector A: PASS"
            return 0
        else
            log_error "Vector A: FAIL (exit code: $exit_code)"
            return 1
        fi
    else
        log_warn "Vector A script not found. Skipping."
        return 2
    fi
}

# Run Vector B: Brain Desync
run_vector_b() {
    log_info "Starting Vector B: Brain Desync Test..."
    
    if [[ -f "$SCRIPT_DIR/vector_b_brain_desync.sh" ]]; then
        bash "$SCRIPT_DIR/vector_b_brain_desync.sh" --duration="$DURATION" --output="$OUTPUT_DIR"
        local exit_code=$?
        
        if [[ $exit_code -eq 0 ]]; then
            log_success "Vector B: PASS"
            return 0
        else
            log_error "Vector B: FAIL (exit code: $exit_code)"
            return 1
        fi
    else
        log_warn "Vector B script not found. Skipping."
        return 2
    fi
}

# Run Vector C: Zero-Copy Jitter
run_vector_c() {
    log_info "Starting Vector C: Zero-Copy Jitter Audit..."
    
    if [[ -f "$SCRIPT_DIR/vector_c_jitter_audit.sh" ]]; then
        bash "$SCRIPT_DIR/vector_c_jitter_audit.sh" --duration="$DURATION" --output="$OUTPUT_DIR"
        local exit_code=$?
        
        if [[ $exit_code -eq 0 ]]; then
            log_success "Vector C: PASS"
            return 0
        else
            log_error "Vector C: FAIL (exit code: $exit_code)"
            return 1
        fi
    else
        log_warn "Vector C script not found. Skipping."
        return 2
    fi
}

# Generate final report
generate_report() {
    local overall_status="$1"
    
    log_info "Generating test report..."
    
    # Update manifest
    jq --arg status "$overall_status" \
       '.status = $status | .end_timestamp = "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"' \
       "$OUTPUT_DIR/test_manifest.json" > "$OUTPUT_DIR/test_manifest.tmp"
    mv "$OUTPUT_DIR/test_manifest.tmp" "$OUTPUT_DIR/test_manifest.json"
    
    if [[ "$REPORT_FORMAT" == "json" ]]; then
        # JSON report
        local report_file="$OUTPUT_DIR/stress_test_report.json"
        
        cat > "$report_file" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "duration_sec": $DURATION,
    "overall_status": "$overall_status",
    "vectors": {
        "telemetry_burst": $(cat "$OUTPUT_DIR/vector_a_results.json" 2>/dev/null || echo '{"status":"SKIPPED"}'),
        "brain_desync": $(cat "$OUTPUT_DIR/vector_b_results.json" 2>/dev/null || echo '{"status":"SKIPPED"}'),
        "zero_copy_jitter": $(cat "$OUTPUT_DIR/vector_c_results.json" 2>/dev/null || echo '{"status":"SKIPPED"}')
    }
}
EOF
        
        log_success "JSON report generated: $report_file"
        cat "$report_file" | jq '.'
    else
        # Text report
        echo ""
        echo "======================================"
        echo "  STRESS TEST REPORT"
        echo "======================================"
        echo "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
        echo "Duration: ${DURATION}s"
        echo "Overall Status: $overall_status"
        echo "======================================"
    fi
}

# Main execution
main() {
    print_banner
    echo ""
    
    check_prerequisites
    init_test_env
    
    local vector_a_status=2  # 0=PASS, 1=FAIL, 2=SKIPPED
    local vector_b_status=2
    local vector_c_status=2
    
    # Run requested vectors
    case "$VECTOR" in
        all)
            run_vector_a && vector_a_status=0 || vector_a_status=1
            run_vector_b && vector_b_status=0 || vector_b_status=1
            run_vector_c && vector_c_status=0 || vector_c_status=1
            ;;
        telemetry)
            run_vector_a && vector_a_status=0 || vector_a_status=1
            ;;
        brain)
            run_vector_b && vector_b_status=0 || vector_b_status=1
            ;;
        jitter)
            run_vector_c && vector_c_status=0 || vector_c_status=1
            ;;
        *)
            log_error "Unknown vector: $VECTOR"
            exit 1
            ;;
    esac
    
    # Determine overall status
    local overall_status="PASS"
    if [[ $vector_a_status -eq 1 ]] || [[ $vector_b_status -eq 1 ]] || [[ $vector_c_status -eq 1 ]]; then
        overall_status="FAIL"
    fi
    
    generate_report "$overall_status"
    
    echo ""
    if [[ "$overall_status" == "PASS" ]]; then
        log_success "ALL TESTS PASSED - SYSTEM READY FOR PRODUCTION"
        exit 0
    else
        log_error "TESTS FAILED - SYSTEM NOT READY FOR PRODUCTION"
        exit 1
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --vector=*)
            VECTOR="${1#*=}"
            shift
            ;;
        --duration=*)
            DURATION="${1#*=}"
            shift
            ;;
        --report=*)
            REPORT_FORMAT="${1#*=}"
            shift
            ;;
        --output=*)
            OUTPUT_DIR="${1#*=}"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

main
