import sys
import time
import socket
import requests
import os
from datetime import datetime, timedelta
from dotenv import load_dotenv

# Load Env
load_dotenv()
TIINGO_KEY = os.getenv("TIINGO_API_KEY")

# Configuration
QUESTDB_HOST = "localhost"
QUESTDB_PORT = 9009
QUESTDB_TABLE = "ohlcv_1min"
SYMBOLS = ["btcusd", "ethusd"]  # Tiingo format
START_DATE = datetime(2019, 1, 1)
END_DATE = datetime(2019, 10, 1)  # Stop where Binance US picks up

# Tiingo Constants
TIINGO_API = "https://api.tiingo.com/tiingo/crypto/prices"


def get_tiingo_data(symbol, date_str):
    params = {
        "tickers": symbol,
        "startDate": date_str,
        "endDate": date_str,  # Daily chunks to be safe with limits
        "resampleFreq": "1min",
        "token": TIINGO_KEY,
    }
    try:
        resp = requests.get(TIINGO_API, params=params, timeout=10)
        resp.raise_for_status()
        return resp.json()
    except Exception as e:
        print(f"❌ Error fetching {symbol} at {date_str}: {e}")
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

    current_date = START_DATE

    while current_date < END_DATE:
        date_str = current_date.strftime("%Y-%m-%d")

        for symbol in SYMBOLS:
            # QuestDB Symbol: BTC-USDT
            qdb_symbol = f"{symbol[:3].upper()}-USDT"

            data = get_tiingo_data(symbol, date_str)

            if not data:
                print(f"⚠️ No data for {symbol} on {date_str}")
                continue

            # Tiingo returns details in data[0]['priceData'] usually?
            # Or data is list of price objects if 'tickers' is single?
            # Let's check structure from previous output:
            # Output was list of dicts: [{'date': '...', 'open': ...}]
            # So data IS the list of candles.

            # Tiingo structure for single ticker request is list of objects.
            # If multiple tickers, it's different. We request one by one.

            # Check if inner 'priceData' exists
            candles = data
            if isinstance(data, list) and len(data) > 0 and "priceData" in data[0]:
                # This happens if metadata is included (rare for this endpoint params but possible)
                pass

            ilp_buffer = ""
            for k in candles:
                # k = {'date': '2019-01-02T00:00:00+00:00', 'open': 3847.16...}
                ts_str = k.get("date")
                if not ts_str:
                    continue

                # Parse ISO timestamp
                # 2019-01-02T00:00:00+00:00
                try:
                    dt = datetime.fromisoformat(ts_str)
                    ts_nanos = int(dt.timestamp() * 1_000_000_000)
                except ValueError:
                    # Generic parse
                    continue

                open_p = k.get("open")
                high_p = k.get("high")
                low_p = k.get("low")
                close_p = k.get("close")
                vol = k.get("volume")

                line = f"{QUESTDB_TABLE},symbol={qdb_symbol},exchange=TIINGO open={open_p},high={high_p},low={low_p},close={close_p},volume={vol} {ts_nanos}\n"
                ilp_buffer += line

            if ilp_buffer:
                try:
                    sock.sendall(ilp_buffer.encode("utf-8"))
                except Exception as e:
                    print(f"❌ Socket Error: {e}")
                    sock = connect_ilp()

        sys.stdout.write(f"\r✅ Processed {date_str}")
        sys.stdout.flush()

        current_date += timedelta(days=1)
        # Tiingo Rate Limit: 50 requests/hour? 1000/day?
        # Free tier: 500 requests/hour.
        # We process 1 day per request per symbol = 2 reqs.
        # 365 days = 730 reqs.
        # Should sleep a bit.
        time.sleep(0.5)

    print("\n🎉 Tiingo Backfill Complete.")
    sock.close()


if __name__ == "__main__":
    run_backfill()
