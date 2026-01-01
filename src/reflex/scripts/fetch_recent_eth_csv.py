import sys
import time
import requests
import csv
from datetime import datetime, timedelta

# Config
PAIR = "ETH-USD"
SYMBOL_OUT = "ETH-USDT"
# Resuming from 2021-08-28 (Day after last fetched data)
START_DT = datetime(2021, 8, 28)
END_DT = datetime.now()
OUTPUT_FILE = "temp_eth_data/ETH_2020_2025.csv"

# Coinbase API
CB_URL = "https://api.exchange.coinbase.com/products/{}/candles"
GRANULARITY = 60  # 1 minute


def get_coinbase_candles(pair, start, end):
    params = {
        "start": start.isoformat(),
        "end": end.isoformat(),
        "granularity": GRANULARITY,
    }
    try:
        # 10 req/s limit. 0.15s sleep is safe (~6.6 req/s)
        time.sleep(0.15)
        resp = requests.get(CB_URL.format(pair), params=params, timeout=10)

        if resp.status_code == 429:
            print("⏳ Rate Limit (429). Sleeping 5s...")
            time.sleep(5)
            # Retry once
            return get_coinbase_candles(pair, start, end)

        if resp.status_code != 200:
            print(f"❌ Coinbase Error {resp.status_code}: {resp.text}")
            return []

        data = resp.json()
        return data  # [time, low, high, open, close, volume]

    except Exception as e:
        print(f"❌ Error fetching {pair} at {start}: {e}")
        return []


def run_fetch():
    print(f"🚀 Resuming Coinbase Fetch for {PAIR} ({START_DT} -> {END_DT})")
    print(f"💾 Appending to {OUTPUT_FILE}")

    # Open CSV for appending
    # Format: ts,low,high,open,close,volume (Standard Coinbase output order)
    with open(OUTPUT_FILE, "a", newline="") as f:
        writer = csv.writer(f)
        # Skip header for append
        # writer.writerow(["ts", "low", "high", "open", "close", "volume"])

        current_start = START_DT
        total_rows = 0

        while current_start < END_DT:
            # Max 300 candles per req = 300 minutes = 5 hours
            chunk_end = current_start + timedelta(hours=5)
            # Ensure we don't go into future or past END_DT
            if chunk_end > END_DT:
                chunk_end = END_DT

            candles = get_coinbase_candles(PAIR, current_start, chunk_end)

            if candles:
                # Coinbase returns newest first. We want to write chronologically?
                # Actually, appending newest-first blocks results in:
                # [Block 1 New->Old] [Block 2 New->Old]...
                # This is messy for sorted writing.
                # However, deep_backfill_crypto.py handles sorting (or I can sort at end).
                # Better: Coinbase returns [Newest ... Oldest] for the requested window.
                # If I request Window 1 (2020-01 to 2020-01+5h), the response is [1+5h ... 1].
                # If I simply reverse `candles` in memory, I get [1 ... 1+5h].
                # Then I write that to file.
                # Then Move to Window 2.
                # Result: File is chronological.

                candles.reverse()
                writer.writerows(candles)
                total_rows += len(candles)
                f.flush()

            sys.stdout.write(
                f"\r✅ {current_start.date()} | Fetched: {len(candles)} | Total: {total_rows:,}"
            )
            sys.stdout.flush()

            current_start = chunk_end

    print("\n🎉 Fetch Complete.")


if __name__ == "__main__":
    run_fetch()
