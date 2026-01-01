import sys
import time
import socket
import requests
import os
from datetime import datetime, timedelta
from dotenv import load_dotenv

load_dotenv()

# Configuration
QUESTDB_HOST = "localhost"
QUESTDB_PORT = 9009
QUESTDB_TABLE = "ohlcv_1min"

# Coinbase Pairs
PAIRS = {"BTC-USD": "BTC-USDT", "ETH-USD": "ETH-USDT"}

# Coinbase Limits
CB_URL = "https://api.exchange.coinbase.com/products/{}/candles"
MAX_CANDLES = 300
GRANULARITY = 60  # 1 minute

START_DT = datetime(2016, 2, 15)
# 2019-01-01 is where we stop to meet the Tiingo data
END_DT = datetime(2019, 1, 1)


def connect_ilp():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((QUESTDB_HOST, QUESTDB_PORT))
        return sock
    except Exception as e:
        print(f"❌ Could not connect to QuestDB ILP: {e}")
        sys.exit(1)


def get_coinbase_candles(pair, start, end):
    params = {
        "start": start.isoformat(),
        "end": end.isoformat(),
        "granularity": GRANULARITY,
    }
    try:
        # Coinbase rate limit is public but stricter (10 req/sec usually, but standard is 3-5 safe)
        time.sleep(0.4)

        resp = requests.get(CB_URL.format(pair), params=params, timeout=10)

        if resp.status_code == 429:
            print("⏳ Rate Limit (429). Sleeping 5s...")
            time.sleep(5)
            return []

        if resp.status_code != 200:
            print(f"❌ Coinbase Error {resp.status_code}: {resp.text}")
            return []

        data = resp.json()
        return data  # List of [time, low, high, open, close, volume]

    except Exception as e:
        print(f"❌ Error fetching {pair} at {start}: {e}")
        return []


def run_backfill():
    sock = connect_ilp()
    print(f"🔌 Connected to QuestDB ILP at {QUESTDB_HOST}:{QUESTDB_PORT}")

    for cb_pair, q_symbol in PAIRS.items():
        print(f"\n🚀 Starting Coinbase Backfill for {cb_pair} -> {q_symbol}...")

        if "ETH" in cb_pair:
            # ETH-USD started Aug 2016 on GDAX/Coinbase approx
            target_start = datetime(2016, 5, 1)  # Safe start
        else:
            target_start = START_DT

        current_start = target_start
        total_ingested = 0

        while current_start < END_DT:
            # Coinbase request covers: Start to End. Max 300 candles = 300 minutes = 5 hours.
            # We'll do 4 hour chunks to be safe.
            chunk_end = current_start + timedelta(hours=4)
            if chunk_end > END_DT:
                chunk_end = END_DT

            candles = get_coinbase_candles(cb_pair, current_start, chunk_end)

            if candles:
                ilp_buffer = ""
                for c in candles:
                    # [time, low, high, open, close, volume]
                    # Coinbase time is Unix Epoch
                    ts_sec = c[0]
                    low_p = c[1]
                    high_p = c[2]
                    open_p = c[3]
                    close_p = c[4]
                    vol = c[5]

                    ts_nanos = int(ts_sec * 1_000_000_000)

                    line = f"{QUESTDB_TABLE},symbol={q_symbol},exchange=COINBASE open={open_p},high={high_p},low={low_p},close={close_p},volume={vol} {ts_nanos}\n"
                    ilp_buffer += line

                try:
                    sock.sendall(ilp_buffer.encode("utf-8"))
                    total_ingested += len(candles)
                except Exception as e:
                    print(f"❌ Socket Error: {e}")
                    sock = connect_ilp()

            sys.stdout.write(
                f"\r      ✅ {current_start.date()} | Ingested: {len(candles) if candles else 0} | Total: {total_ingested:,}"
            )
            sys.stdout.flush()

            current_start = chunk_end

    print("\n🎉 Coinbase Backfill Complete.")
    sock.close()


if __name__ == "__main__":
    run_backfill()
