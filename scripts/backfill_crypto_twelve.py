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

API_KEY = os.getenv("TWELVEDATA_API_KEY")
BASE_URL = "https://api.twelvedata.com/time_series"

# TwelveData symbols: BTC/USD, ETH/USD
SYMBOLS = ["BTC/USD", "ETH/USD"]
SYMBOL_MAP = {"BTC/USD": "BTC-USDT", "ETH/USD": "ETH-USDT"}

# TwelveData allows fetching by "date" (specific day) or "start_date"/"end_date".
# Max outputsize is 5000. 1 day = 1440 mins.
START_DT = datetime(2015, 1, 1)
END_DT = datetime(2019, 9, 1)


def connect_ilp():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((QUESTDB_HOST, QUESTDB_PORT))
        return sock
    except Exception as e:
        print(f"❌ Could not connect to QuestDB ILP: {e}")
        sys.exit(1)


def get_twelve_data(symbol, start_date, end_date):
    params = {
        "symbol": symbol,
        "interval": "1min",
        "start_date": start_date.strftime("%Y-%m-%d %H:%M:%S"),
        "end_date": end_date.strftime("%Y-%m-%d %H:%M:%S"),
        "apikey": API_KEY,
        "outputsize": 5000,
        "order": "ASC",
    }
    try:
        resp = requests.get(BASE_URL, params=params, timeout=15)
        data = resp.json()

        if data.get("status") == "error":
            print(f"⚠️ TwelveData Error for {symbol}: {data.get('message')}")
            return []

        return data.get("values", [])
    except Exception as e:
        print(f"❌ Request Error: {e}")
        return []


def run_backfill():
    sock = connect_ilp()
    print(f"🔌 Connected to QuestDB ILP at {QUESTDB_HOST}:{QUESTDB_PORT}")

    for symbol in SYMBOLS:
        q_symbol = SYMBOL_MAP[symbol]
        print(f"\n🚀 Starting TwelveData Backfill for {symbol} -> {q_symbol}...")

        current_dt = START_DT
        total_ingested = 0

        while current_dt < END_DT:
            # Fetch 2 days at a time (approx 2880 mins < 5000 limit)
            next_dt = current_dt + timedelta(days=2)
            if next_dt > END_DT:
                next_dt = END_DT

            print(f"   📅 Fetching {current_dt.date()} to {next_dt.date()}")

            candles = get_twelve_data(symbol, current_dt, next_dt)

            if not candles:
                # If no data, minimal sleep and move on
                current_dt = next_dt
                time.sleep(0.5)
                continue

            ilp_buffer = ""
            for c in candles:
                # c = {'datetime': '2019-09-03 15:59:00', 'open': '...', ...}
                try:
                    ts_str = c["datetime"]
                    dt = datetime.strptime(ts_str, "%Y-%m-%d %H:%M:%S")
                    ts_nanos = int(dt.timestamp() * 1_000_000_000)

                    open_p = c["open"]
                    high_p = c["high"]
                    low_p = c["low"]
                    close_p = c["close"]
                    vol = c.get("volume", 0)

                    line = f"{QUESTDB_TABLE},symbol={q_symbol},exchange=TWELVEDATA open={open_p},high={high_p},low={low_p},close={close_p},volume={vol} {ts_nanos}\n"
                    ilp_buffer += line
                except Exception:
                    continue

            if ilp_buffer:
                try:
                    sock.sendall(ilp_buffer.encode("utf-8"))
                    total_ingested += len(candles)
                    sys.stdout.write(
                        f"\r      ✅ Ingested {len(candles)} rows | Total: {total_ingested:,}"
                    )
                    sys.stdout.flush()
                except Exception as e:
                    print(f"❌ Socket Error: {e}")
                    sock = connect_ilp()

            current_dt = next_dt
            # Rate limit safety (TwelveData free tier is 8/min or similar?)
            # 800 credits/day usually?
            time.sleep(1.0)

    print("\n🎉 TwelveData Backfill Complete.")
    sock.close()


if __name__ == "__main__":
    run_backfill()
