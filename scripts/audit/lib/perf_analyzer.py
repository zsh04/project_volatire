#!/usr/bin/env python3
"""
Helper script to parse 'perf stat -I' output and calculate jitter metrics.
Used by vector_c_jitter_audit.sh.
"""

import sys
import argparse
import json
import statistics
import re

def parse_perf_output(filepath):
    data = {
        'cycles': [],
        'cache_misses': [],
        'cache_references': []
    }

    # Regex to handle perf -I output
    # Example: "     1.001056525          1,234,567      cycles"
    # Capture: Timestamp, Count, Event
    pattern = re.compile(r'^\s*(\d+\.\d+)\s+([0-9,]+|<not counted>|<not supported>)\s+([\w-]+)')

    try:
        with open(filepath, 'r') as f:
            for line in f:
                line = line.strip()
                match = pattern.search(line)
                if not match:
                    continue

                count_str = match.group(2)
                event = match.group(3)

                if count_str.startswith('<'):
                    continue

                count = int(count_str.replace(',', ''))

                if event == 'cycles' or event == 'cpu-cycles':
                    data['cycles'].append(count)
                elif event == 'cache-misses':
                    data['cache_misses'].append(count)
                elif event == 'cache-references':
                    data['cache_references'].append(count)

    except FileNotFoundError:
        print(json.dumps({"error": f"File not found: {filepath}"}))
        sys.exit(1)

    return data

def calculate_stats(data, interval_sec):
    results = {}

    # Cache Stats
    total_misses = sum(data['cache_misses'])
    total_refs = sum(data['cache_references'])

    if total_refs > 0:
        results['cache_miss_rate_pct'] = (total_misses / total_refs) * 100.0
    else:
        results['cache_miss_rate_pct'] = 0.0

    # Jitter (Std Dev of Cycles)
    cycles = data['cycles']
    if len(cycles) > 1:
        # Standard Deviation of Cycle Counts
        std_dev_cycles = statistics.stdev(cycles)
        mean_cycles = statistics.mean(cycles)

        # Calculate approximate frequency from mean cycles per interval
        # Freq (Hz) = MeanCycles / Interval(s)
        if interval_sec > 0 and mean_cycles > 0:
            freq_hz = mean_cycles / interval_sec

            # Convert Std Dev (Cycles) to Std Dev (Microseconds)
            # StdDev(t) = StdDev(Cycles) / Freq
            # result in seconds, convert to us (* 1e6)
            std_dev_s = std_dev_cycles / freq_hz
            results['ooda_jitter_std_dev_us'] = std_dev_s * 1_000_000

            # Max Spike (Max deviation from mean)
            max_dev_cycles = max([abs(c - mean_cycles) for c in cycles])
            results['ooda_jitter_max_us'] = (max_dev_cycles / freq_hz) * 1_000_000
        else:
             results['ooda_jitter_std_dev_us'] = 0.0
             results['ooda_jitter_max_us'] = 0.0

    else:
        results['ooda_jitter_std_dev_us'] = 0.0
        results['ooda_jitter_max_us'] = 0.0

    return results

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', required=True, help='Input perf stats file')
    parser.add_argument('--interval', type=float, default=0.1, help='Interval in seconds')
    args = parser.parse_args()

    data = parse_perf_output(args.input)
    stats = calculate_stats(data, args.interval)

    # Print JSON to stdout
    print(json.dumps(stats, indent=2))

if __name__ == '__main__':
    main()
