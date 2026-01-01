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

ALPACA_KEY = os.getenv("ALPACA_API_KEY")
ALPACA_SECRET = os.getenv("ALPACA_API_SECRET")
# Alpaca Crypto History Endpoint
ALPACA_URL = "https://data.alpaca.markets/v1beta3/crypto/us/bars"

SYMBOLS = ["BTC/USD", "ETH/USD"]  # Alpaca Format
# QuestDB Map: BTC/USD -> BTC-USDT
SYMBOL_MAP = {"BTC/USD": "BTC-USDT", "ETH/USD": "ETH-USDT"}

START_DT = datetime(2016, 1, 1)  # Alpaca crypto starts ~2016
END_DT = datetime(2019, 9, 1)


def connect_ilp():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((QUESTDB_HOST, QUESTDB_PORT))
        return sock
    except Exception as e:
        print(f"❌ Could not connect to QuestDB ILP: {e}")
        sys.exit(1)


def get_alpaca_bars(symbol, start, end, page_token=None):
    headers = {"APCA-API-KEY-ID": ALPACA_KEY, "APCA-API-SECRET-KEY": ALPACA_SECRET}
    params = {
        "symbols": symbol,
        "timeframe": "1Min",
        "start": start.isoformat() + "Z",
        "end": end.isoformat() + "Z",
        "limit": 10000,
        "sort": "asc",
    }
    if page_token:
        params["page_token"] = page_token

    resp = requests.get(ALPACA_URL, headers=headers, params=params)
    if resp.status_code != 200:
        print(f"❌ Alpaca Error {resp.status_code}: {resp.text}")
        return {}, None

    data = resp.json()
    bars = data.get("bars", {})
    next_token = data.get("next_page_token")
    return bars, next_token


def run_backfill():
    sock = connect_ilp()
    print(f"🔌 Connected to QuestDB ILP at {QUESTDB_HOST}:{QUESTDB_PORT}")

    for symbol in SYMBOLS:
        q_symbol = SYMBOL_MAP[symbol]
        print(f"\n🚀 Starting Alpaca Backfill for {symbol} -> {q_symbol}...")

        current_start = START_DT
        total_ingested = 0

        # Chunk by Month to avoid huge requests/token loops
        while current_start < END_DT:
            chunk_end = current_start + timedelta(days=30)
            if chunk_end > END_DT:
                chunk_end = END_DT

            print(f"   📅 Fetching chunk: {current_start.date()} to {chunk_end.date()}")

            page_token = None
            while True:
                bars_dict, next_token = get_alpaca_bars(
                    symbol, current_start, chunk_end, page_token
                )
                bars = bars_dict.get(symbol, [])

                if not bars:
                    # No data for this chunk/page
                    if next_token:
                        page_token = next_token
                        continue
                    else:
                        break

                ilp_buffer = ""
                for b in bars:
                    # b = {'t': '2021-01...', 'o': ..., 'h': ..., 'l': ..., 'c': ..., 'v': ...}
                    try:
                        dt = datetime.fromisoformat(b["t"].replace("Z", "+00:00"))
                        ts_nanos = int(dt.timestamp() * 1_000_000_000)

                        open_p = b["o"]
                        high_p = b["h"]
                        low_p = b["l"]
                        close_p = b["c"]
                        vol = b["v"]

                        line = f"{QUESTDB_TABLE},symbol={q_symbol},exchange=ALPACA open={open_p},high={high_p},low={low_p},close={close_p},volume={vol} {ts_nanos}\n"
                        ilp_buffer += line
                    except Exception:
                        continue

                if ilp_buffer:
                    try:
                        sock.sendall(ilp_buffer.encode("utf-8"))
                        total_ingested += len(bars)
                        sys.stdout.write(
                            f"\r      ✅ Ingested {len(bars)} rows | Total: {total_ingested:,}"
                        )
                        sys.stdout.flush()
                    except Exception as e:
                        print(f"❌ Socket Error: {e}")
                        sock = connect_ilp()

                if not next_token:
                    break
                page_token = next_token
                time.sleep(0.1)

            print()  # Newline
            current_start = chunk_end

    print("\n🎉 Alpaca Backfill Complete.")
    sock.close()


if __name__ == "__main__":
    run_backfill()
