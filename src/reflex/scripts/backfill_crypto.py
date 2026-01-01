import os
import time
import requests
import pandas as pd
from datetime import datetime, timedelta, timezone
from questdb.ingress import Sender, TimestampNanos
import sys

# --- Configuration ---
SYMBOLS = ["BTCUSDT", "ETHUSDT"]
START_DATE = "2020-01-01"  # Start of our sim window
END_DATE = "2020-04-01"  # Reduced for Verification Speed (Covering March Crash)
QUESTDB_HOST = os.getenv("QUESTDB_HOST", "localhost")
QUESTDB_ILP_PORT = os.getenv("QUESTDB_ILP_PORT", "9009")
CONF_STR = f"tcp::addr={QUESTDB_HOST}:{QUESTDB_ILP_PORT};"

# Binance Public API
BASE_URL = "https://api.binance.us/api/v3/klines"


def fetch_batch(symbol, start_ts_ms, limit=1000):
    params = {
        "symbol": symbol,
        "interval": "1m",
        "startTime": start_ts_ms,
        "limit": limit,
    }
    try:
        resp = requests.get(BASE_URL, params=params, timeout=10)
        resp.raise_for_status()
        return resp.json()
    except Exception as e:
        print(f"⚠️ API Error: {e}")
        time.sleep(1)
        return []


def run_backfill():
    print(f"🚀 Starting Crypto Backfill: {SYMBOLS}")
    print(f"📡 QuestDB: {QUESTDB_HOST}:{QUESTDB_ILP_PORT}")

    # Connect to QuestDB
    try:
        with Sender.from_conf(CONF_STR) as sender:
            for symbol in SYMBOLS:
                print(f"📥 Processing {symbol}...")

                # Setup Time Window
                start_dt = datetime.strptime(START_DATE, "%Y-%m-%d").replace(
                    tzinfo=timezone.utc
                )
                end_dt = datetime.strptime(END_DATE, "%Y-%m-%d").replace(
                    tzinfo=timezone.utc
                )

                current_ts = int(start_dt.timestamp() * 1000)
                end_ts = int(end_dt.timestamp() * 1000)

                total_records = 0

                while current_ts < end_ts:
                    batch = fetch_batch(symbol, current_ts)

                    if not batch:
                        print("🚫 No data returned, stopping or retrying...")
                        break

                    for kline in batch:
                        # Binance Kline:
                        # [0: Open Time, 1: Open, 2: High, 3: Low, 4: Close, 5: Vol, 6: Close Time...]
                        ts_ms = kline[0]
                        open_px = float(kline[1])
                        high_px = float(kline[2])
                        low_px = float(kline[3])
                        close_px = float(kline[4])
                        volume = float(kline[5])

                        # Ingest Row
                        sender.row(
                            "ohlcv_1min",
                            symbols={
                                "symbol": symbol.replace(
                                    "USDT", "-USDT"
                                )  # Match internal format
                            },
                            columns={
                                "open": open_px,
                                "high": high_px,
                                "low": low_px,
                                "close": close_px,
                                "volume": volume,
                            },
                            at=TimestampNanos.from_datetime(
                                datetime.fromtimestamp(ts_ms / 1000, timezone.utc)
                            ),
                        )

                    # Flush Batch
                    sender.flush()

                    count = len(batch)
                    total_records += count

                    # Update Cursor
                    last_ts = batch[-1][0]
                    # Binance returns inclusive start, so next batch starts at last_ts + 1m (60000ms)
                    # Actually valid strategy check: if we got < 1000 items, we might be at head.
                    if count < 1000:
                        current_ts = last_ts + 60000
                    else:
                        current_ts = last_ts + 60000

                    if total_records % 10000 == 0:
                        print(
                            f"   Using {symbol}: {total_records} rows... (Last: {datetime.fromtimestamp(current_ts / 1000, timezone.utc)})"
                        )

                    # Rate Limit
                    time.sleep(0.1)

                print(f"✅ {symbol} Complete. Total Rows: {total_records}")

    except Exception as e:
        print(f"❌ Critical Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    run_backfill()
