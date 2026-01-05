#!/usr/bin/env python3
"""
Directive-84: Jitter Analysis Script
Analyzes the phase6_stress.sh JSON report to extract critical metrics
and verify acceptance criteria.
"""

import json
import sys
import argparse
from typing import Dict, Any

# ANSI Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'

def load_report(filepath: str) -> Dict[str, Any]:
    """Load the JSON report from file"""
    try:
        with open(filepath, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"{RED}Error: Report file not found: {filepath}{NC}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"{RED}Error: Invalid JSON in report: {e}{NC}")
        sys.exit(1)

def analyze_vector_a(data: Dict[str, Any]) -> bool:
    """Analyze Telemetry Burst results"""
    print(f"\n{BLUE}═══ Vector A: Telemetry Burst ═══{NC}")
    
    vector_a = data.get('vectors', {}).get('telemetry_burst', {})
    if vector_a.get('status') == 'SKIPPED':
        print(f"{YELLOW}⊘ SKIPPED{NC}")
        return True
    
    fps = vector_a.get('stress_fps', 0)
    latency = vector_a.get('worker_latency_p99_ms', 0)
    jank = vector_a.get('jank_count', 0)
    
    print(f"  FPS (stress):          {fps:.1f} fps")
    print(f"  Worker Latency (p99):  {latency:.2f} ms")
    print(f"  Jank Events:           {jank}")
    
    passed = True
    if fps < 60:
        print(f"  {RED}✗ FPS below threshold (< 60){NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ FPS acceptable{NC}")
    
    if latency >= 4.0:
        print(f"  {RED}✗ Worker latency too high (>= 4ms){NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ Worker latency acceptable{NC}")
    
    if jank > 0:
        print(f"  {RED}✗ Main thread jank detected{NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ No main thread jank{NC}")
    
    return passed

def analyze_vector_b(data: Dict[str, Any]) -> bool:
    """Analyze Brain Desync results"""
    print(f"\n{BLUE}═══ Vector B: Brain Desync ═══{NC}")
    
    vector_b = data.get('vectors', {}).get('brain_desync', {})
    if vector_b.get('status') == 'SKIPPED':
        print(f"{YELLOW}⊘ SKIPPED{NC}")
        return True
    
    out_of_order = vector_b.get('out_of_order_events', 0)
    accuracy = vector_b.get('gemma_audit_accuracy_pct', 0)
    lag_warnings = vector_b.get('cognitive_lag_warnings', 0)
    brain_latency = vector_b.get('brain_delay_ms', 500)
    
    print(f"  GSID Out-of-Order:     {out_of_order}")
    print(f"  Gemma Accuracy:        {accuracy:.1f}%")
    print(f"  Cognitive Lag Warns:   {lag_warnings}")
    print(f"  Brain Delay (applied): {brain_latency} ms")
    
    passed = True
    
    # BRUTAL TRUTH #1: GSID Integrity
    if out_of_order > 0:
        print(f"  {RED}✗ CRITICAL: GSID ordering violated ({out_of_order} events){NC}")
        print(f"  {RED}  → Race condition in Priority Queue (D-79){NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ GSID sequencing intact{NC}")
    
    if accuracy < 95.0:
        print(f"  {RED}✗ Gemma audit accuracy below 95%{NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ Gemma audit accuracy acceptable{NC}")
    
    if lag_warnings == 0:
        print(f"  {YELLOW}⚠ No cognitive lag warnings (expected ≥1){NC}")
    else:
        print(f"  {GREEN}✓ Cognitive lag detection functional{NC}")
    
    return passed

def analyze_vector_c(data: Dict[str, Any]) -> bool:
    """Analyze Zero-Copy Jitter results"""
    print(f"\n{BLUE}═══ Vector C: Zero-Copy Jitter ═══{NC}")
    
    vector_c = data.get('vectors', {}).get('zero_copy_jitter', {})
    if vector_c.get('status') == 'SKIPPED':
        print(f"{YELLOW}⊘ SKIPPED{NC}")
        return True
    
    jitter_us = vector_c.get('ooda_jitter_max_us', 0)
    cache_miss_pct = vector_c.get('cache_miss_rate_pct', 0)
    interrupts = vector_c.get('core_interrupts', 0)
    bandwidth = vector_c.get('historian_bandwidth_mb_min', 0)
    
    print(f"  OODA Jitter (max):     {jitter_us} μs")
    print(f"  Cache Miss Rate:       {cache_miss_pct:.2f}%")
    print(f"  Core Interrupts:       {interrupts}")
    print(f"  Historian Bandwidth:   {bandwidth:.1f} MB/min")
    
    passed = True
    
    if jitter_us >= 50:
        print(f"  {RED}✗ OODA jitter too high (>= 50μs){NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ OODA jitter acceptable{NC}")
    
    if cache_miss_pct >= 1.0:
        print(f"  {RED}✗ Cache pollution detected (>= 1%){NC}")
        print(f"  {RED}  → Archiver stealing CPU cycles{NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ Cache integrity maintained{NC}")
    
    # BRUTAL TRUTH #3: The Sentry's Word
    if interrupts > 0:
        print(f"  {RED}✗ CRITICAL: Core interrupts detected ({interrupts}){NC}")
        print(f"  {RED}  → Kernel isolation compromised (D-80){NC}")
        passed = False
    else:
        print(f"  {GREEN}✓ Core isolation verified{NC}")
    
    return passed

def analyze_overall(data: Dict[str, Any]) -> bool:
    """Analyze overall test results"""
    print(f"\n{BLUE}═══════════════════════════════════{NC}")
    print(f"{BLUE}     OVERALL SYSTEM STATUS{NC}")
    print(f"{BLUE}═══════════════════════════════════{NC}")
    
    overall_status = data.get('overall_status', 'UNKNOWN')
    duration = data.get('duration_sec', 0)
    timestamp = data.get('timestamp', 'unknown')
    
    print(f"  Test Duration:  {duration}s")
    print(f"  Timestamp:      {timestamp}")
    print(f"  Status:         {overall_status}")
    
    return overall_status == "PASS"

def main():
    parser = argparse.ArgumentParser(description='Analyze Phase 6 stress test results')
    parser.add_argument('--input', required=True, help='Path to JSON report')
    parser.add_argument('--verbose', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    print(f"{BLUE}╔═══════════════════════════════════════════╗{NC}")
    print(f"{BLUE}║  D-84: FORENSIC AUDIT ANALYSIS           ║{NC}")
    print(f"{BLUE}╚═══════════════════════════════════════════╝{NC}")
    
    # Load report
    data = load_report(args.input)
    
    # Analyze each vector
    vector_a_pass = analyze_vector_a(data)
    vector_b_pass = analyze_vector_b(data)
    vector_c_pass = analyze_vector_c(data)
    overall_pass = analyze_overall(data)
    
    # Final verdict
    print(f"\n{BLUE}═══════════════════════════════════{NC}")
    print(f"{BLUE}     ACCEPTANCE CRITERIA{NC}")
    print(f"{BLUE}═══════════════════════════════════{NC}")
    
    all_passed = vector_a_pass and vector_b_pass and vector_c_pass and overall_pass
    
    if all_passed:
        print(f"\n{GREEN}✓ ALL VECTORS PASSED{NC}")
        print(f"{GREEN}✓ SYSTEM READY FOR PRODUCTION{NC}")
        sys.exit(0)
    else:
        print(f"\n{RED}✗ ONE OR MORE VECTORS FAILED{NC}")
        print(f"{RED}✗ SYSTEM NOT READY FOR PRODUCTION{NC}")
        
        print(f"\n{YELLOW}Recommended Actions:{NC}")
        if not vector_a_pass:
            print(f"  • Optimize HUD rendering pipeline")
            print(f"  • Reduce worker message overhead")
        if not vector_b_pass:
            print(f"  • Fix Event Sequencer race condition")
            print(f"  • Prune Brain system prompt for latency")
        if not vector_c_pass:
            print(f"  • Verify kernel isolation parameters")
            print(f"  • Reduce Archiver CPU contention")
        
        sys.exit(1)

if __name__ == '__main__':
    main()
