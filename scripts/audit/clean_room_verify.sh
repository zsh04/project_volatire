#!/usr/bin/env bash
# Directive-84: Clean Room Verification Protocol
# Executes the full stress test suite in a pristine environment

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[CLEAN-ROOM]${NC} $*"; }
log_success() { echo -e "${GREEN}[CLEAN-ROOM]${NC} $*"; }
log_error() { echo -e "${RED}[CLEAN-ROOM]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[CLEAN-ROOM]${NC} $*"; }

# Print banner
print_banner() {
    cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   DIRECTIVE-84: CLEAN ROOM VERIFICATION PROTOCOL         ║
║                                                           ║
║   "The Final Handshake of the Frontier Phase"           ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
}

# Step 1: Clear the environment
clear_environment() {
    log_info "Step 1: Clearing environment (purging ghost states)..."
    
    # Clear shared memory buffers
    log_info "Purging /dev/shm..."
    rm -f /dev/shm/reflex_* 2>/dev/null || true
    
    # Stop any running containers
    if command -v docker-compose &> /dev/null; then
        log_info "Stopping Docker containers..."
        cd "$PROJECT_ROOT"
        docker-compose down 2>/dev/null || true
        
        log_info "Restarting Docker containers..."
        docker-compose up -d 2>/dev/null || true
    else
        log_warn "docker-compose not found, skipping container cleanup"
    fi
    
    # Clear kernel page cache (requires sudo)
    if [[ "$EUID" -eq 0 ]]; then
        log_info "Dropping kernel caches..."
        sync
        echo 3 > /proc/sys/vm/drop_caches
    else
        log_warn "Not running as root, cannot drop kernel caches"
        log_warn "Consider running: sudo sync && sudo sh -c 'echo 3 > /proc/sys/vm/drop_caches'"
    fi
    
    log_success "Environment cleared"
}

# Step 2: Execute the stress suite
execute_stress_suite() {
    log_info "Step 2: Executing Final Forensic Audit (300s duration)..."
    
    local output_file="$PROJECT_ROOT/phase6_final_report.json"
    
    # Run the full suite
    "$SCRIPT_DIR/phase6_stress.sh" \
        --vector=all \
        --duration=300 \
        --report=json \
        | tee "$output_file"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [[ $exit_code -eq 0 ]]; then
        log_success "Stress suite completed successfully"
    else
        log_error "Stress suite failed with exit code $exit_code"
        return 1
    fi
    
    log_info "Report saved to: $output_file"
}

# Step 3: Analyze results
analyze_results() {
    log_info "Step 3: Analyzing jitter vs. load..."
    
    local report_file="$PROJECT_ROOT/phase6_final_report.json"
    
    if [[ ! -f "$report_file" ]]; then
        log_error "Report file not found: $report_file"
        return 1
    fi
    
    # Run analysis script
    if command -v python3 &> /dev/null; then
        python3 "$SCRIPT_DIR/analyze_jitter.py" --input="$report_file"
        local analysis_exit=$?
        
        if [[ $analysis_exit -eq 0 ]]; then
            log_success "Analysis complete: SYSTEM READY FOR PRODUCTION"
            return 0
        else
            log_error "Analysis detected failures: SYSTEM NOT READY"
            return 1
        fi
    else
        log_error "python3 not found, cannot run analysis"
        return 1
    fi
}

# Step 4: Generate Readiness Certificate (D-85)
generate_certificate() {
    log_info "Step 4: Generating Forensic Seal Certificate (D-85)..."
    
    local report_file="$PROJECT_ROOT/phase6_final_report.json"
    local cert_file="$PROJECT_ROOT/PHASE6_READY.json"
    
    if [[ ! -f "$report_file" ]]; then
        log_error "Report file not found: $report_file"
        return 1
    fi
    
    # Generate and sign certificate
    if command -v python3 &> /dev/null; then
        python3 "$SCRIPT_DIR/report_generator.py" \
            --input="$report_file" \
            --output="$cert_file" \
            --sign
        
        local cert_exit=$?
        
        if [[ $cert_exit -eq 0 ]]; then
            log_success "Certificate generated: $cert_file"
            
            # Verify certificate
            python3 "$SCRIPT_DIR/report_generator.py" --verify="$cert_file"
            if [[ $? -eq 0 ]]; then
                log_success "Certificate signature verified"
            fi
            
            return 0
        else
            log_error "Certificate generation failed"
            return 1
        fi
    else
        log_error "python3 not found, cannot generate certificate"
        return 1
    fi
}

# Main execution
main() {
    print_banner
    echo ""
    
    log_info "Clean Room Verification Protocol initiated"
    log_info "This will take approximately 5-6 minutes"
    echo ""
    
    # Confirm with user
    read -p "$(echo -e ${YELLOW}Continue with Clean Room verification? [y/N]:${NC} )" -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warn "Aborted by user"
        exit 0
    fi
    
    # Execute protocol
    clear_environment
    echo ""
    
    execute_stress_suite
    local suite_result=$?
    echo ""
    
    if [[ $suite_result -ne 0 ]]; then
        log_error "Stress suite failed. Aborting analysis."
        exit 1
    fi
    
    analyze_results
    local analysis_result=$?
    echo ""
    
    if [[ $analysis_result -ne 0 ]]; then
        log_error "Analysis failed. Skipping certificate generation."
        log_error "⛔ CLEAN ROOM VERIFICATION: FAILED"
        log_error "🔧 SYSTEM REQUIRES REMEDIATION"
        exit 1
    fi
    
    # D-85: Generate readiness certificate
    generate_certificate
    local cert_result=$?
    
    echo ""
    log_info "═══════════════════════════════════════"
    
    if [[ $cert_result -eq 0 ]]; then
        log_success "🎯 CLEAN ROOM VERIFICATION: PASSED"
        log_success "🔒 FORENSIC SEAL: CERTIFICATE SIGNED"
        log_success "🚀 SYSTEM CLEARED FOR PHASE 7 IGNITION"
        exit 0
    else
        log_error "⛔ CERTIFICATE GENERATION: FAILED"
        log_error "🔧 SYSTEM REQUIRES REMEDIATION"
        exit 1
    fi
}

main "$@"
