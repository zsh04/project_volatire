import sys
import time
import socket
import requests
import os
from datetime import datetime, timedelta

# Configuration
QUESTDB_HOST = "localhost"
QUESTDB_PORT = 9009
QUESTDB_TABLE = "ohlcv_1min"
SYMBOLS = ["BTCUSDT", "ETHUSDT"]
START_DATE = datetime(2015, 1, 1)
END_DATE = datetime.now()

# Binance Constants
BINANCE_API = "https://api.binance.us/api/v3/klines"
LIMIT = 1000


def get_binance_data(symbol, start_ts_ms):
    params = {
        "symbol": symbol,
        "interval": "1m",
        "startTime": start_ts_ms,
        "limit": LIMIT,
    }
    try:
        resp = requests.get(BINANCE_API, params=params, timeout=10)

        if resp.status_code == 451:
            print("❌ Geoblocked (451). Cannot fetch from Binance US/Com.")
            sys.exit(1)

        resp.raise_for_status()
        return resp.json()
    except Exception as e:
        print(f"❌ Error fetching {symbol} at {start_ts_ms}: {e}")
        return []


def connect_ilp():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((QUESTDB_HOST, QUESTDB_PORT))
        return sock
    except Exception as e:
        print(f"❌ Could not connect to QuestDB ILP: {e}")
        sys.exit(1)


def run_backfill():
    sock = connect_ilp()
    print(f"🔌 Connected to QuestDB ILP at {QUESTDB_HOST}:{QUESTDB_PORT}")

    for symbol in SYMBOLS:
        print(f"\n🚀 Starting Backfill for {symbol}...")

        # QuestDB Symbol format: BTC-USDT
        qdb_symbol = f"{symbol[:-4]}-{symbol[-4:]}"  # BTCUSDT -> BTC-USDT

        current_ts = int(START_DATE.timestamp() * 1000)
        end_ts = int(END_DATE.timestamp() * 1000)

        total_records = 0

        while current_ts < end_ts:
            klines = get_binance_data(symbol, current_ts)

            if not klines:
                print(
                    f"⚠️ No data returned for {symbol} at {current_ts}. Moving forward 1 day."
                )
                current_ts += (
                    24 * 60 * 60 * 1000
                )  # Skip day or break? Binance might have gaps before listing.
                if current_ts > end_ts:
                    break
                continue

            ilp_buffer = ""
            last_ts = current_ts

            for k in klines:
                # Binance Kline structure:
                # [0:Open Time, 1:Open, 2:High, 3:Low, 4:Close, 5:Volume, ...]
                ts_ms = k[0]
                open_p = k[1]
                high_p = k[2]
                low_p = k[3]
                close_p = k[4]
                vol = k[5]

                # ILP Line Protocol
                # table,symbol=... open=... timestamp_nanos
                ts_nanos = ts_ms * 1_000_000
                line = f"{QUESTDB_TABLE},symbol={qdb_symbol},exchange=BINANCE open={open_p},high={high_p},low={low_p},close={close_p},volume={vol} {ts_nanos}\n"
                ilp_buffer += line

                last_ts = ts_ms

            # Send Batch
            try:
                sock.sendall(ilp_buffer.encode("utf-8"))
                total_records += len(klines)

                # Progress
                dt_str = datetime.fromtimestamp(last_ts / 1000).strftime("%Y-%m-%d")
                sys.stdout.write(
                    f"\r✅ {symbol}: Processed up to {dt_str} | Total: {total_records:,}"
                )
                sys.stdout.flush()

            except Exception as e:
                print(f"\n❌ Socket Send Error: {e}")
                sock = connect_ilp()  # Reconnect

            # Next Batch
            current_ts = last_ts + 60000  # Next minute

            # Rate Limit Protection
            time.sleep(0.05)

    print("\n🎉 Backfill Complete.")
    sock.close()


if __name__ == "__main__":
    run_backfill()
