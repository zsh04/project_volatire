import sys
import time
import socket
import numpy as np
import pandas as pd
from datetime import datetime
from dotenv import load_dotenv

load_dotenv()

# Configuration
QUESTDB_HOST = "localhost"
QUESTDB_PORT = 9009
QUESTDB_TABLE = "ohlcv_1min"

# Target Gap
START_DT = datetime(2015, 1, 1)
END_DT = datetime(2019, 1, 1)  # Stop before real data starts (approx Jan/Sept 2019)

# Rough starting prices for Jan 2015
START_PRICES = {
    "BTC-USDT": 320.0,
    "ETH-USDT": 1.0,  # ETH didn't exist in Jan 2015, barely launched mid-2015. We'll start it low.
}

# Approx volatility per minute
VOLATILITY = 0.0005


def connect_ilp():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect((QUESTDB_HOST, QUESTDB_PORT))
        return sock
    except Exception as e:
        print(f"❌ Could not connect to QuestDB ILP: {e}")
        sys.exit(1)


def generate_gbm(start_price, n_steps, mu=0, sigma=VOLATILITY):
    dt = 1
    # Simple Geometric Brownian Motion
    returns = np.random.normal(loc=mu * dt, scale=sigma * np.sqrt(dt), size=n_steps)
    price = start_price * (1 + returns).cumprod()
    return price


def run_backfill():
    sock = connect_ilp()
    print(f"🔌 Connected to QuestDB ILP at {QUESTDB_HOST}:{QUESTDB_PORT}")

    # Generate time index
    print(f"⏳ Generating Time Index from {START_DT} to {END_DT}...")
    daterange = pd.date_range(start=START_DT, end=END_DT, freq="1min")
    n_steps = len(daterange)
    print(f"📊 Total Steps: {n_steps:,}")

    for symbol, start_price in START_PRICES.items():
        if symbol == "ETH-USDT":
            # ETH launched July 2015. We will generate from 2015-08-01 to avoid pre-existence confusion
            eth_start = datetime(2015, 8, 7)
            if eth_start > END_DT:
                continue

            # Filter daterange for ETH
            mask = daterange >= eth_start
            sub_dr = daterange[mask]

            print(
                f"\n🚀 Generating Synthetic Data for {symbol} ({len(sub_dr):,} rows)..."
            )
            prices = generate_gbm(start_price, len(sub_dr))
            current_dr = sub_dr
        else:
            print(f"\n🚀 Generating Synthetic Data for {symbol} ({n_steps:,} rows)...")
            prices = generate_gbm(start_price, n_steps)
            current_dr = daterange

        # Batch Send
        batch_size = 10000
        total_sent = 0
        ilp_buffer = ""

        for i, (ts, price) in enumerate(zip(current_dr, prices)):
            # Create synthetic OHLC from the close price "price"
            # Random noise for High/Low
            noise = np.random.uniform(0.999, 1.001, 3)
            close_p = price
            open_p = price * noise[0]
            high_p = max(open_p, close_p) * (1 + abs(noise[1] - 1))
            low_p = min(open_p, close_p) * (1 - abs(noise[2] - 1))
            vol = np.random.uniform(1.0, 100.0)  # Dummy volume

            ts_nanos = int(ts.timestamp() * 1_000_000_000)

            line = f"{QUESTDB_TABLE},symbol={symbol},exchange=SYNTHETIC open={open_p:.5f},high={high_p:.5f},low={low_p:.5f},close={close_p:.5f},volume={vol:.5f} {ts_nanos}\n"
            ilp_buffer += line

            if (i + 1) % batch_size == 0:
                try:
                    sock.sendall(ilp_buffer.encode("utf-8"))
                    total_sent += batch_size
                    ilp_buffer = ""
                    sys.stdout.write(
                        f"\r      ✅ Sent {total_sent:,} / {len(current_dr):,}"
                    )
                    sys.stdout.flush()
                except BrokenPipeError:
                    sock = connect_ilp()

        # Final Flush
        if ilp_buffer:
            sock.sendall(ilp_buffer.encode("utf-8"))

        print(f"\n🎉 {symbol} Synthetic generation complete.")

    sock.close()


if __name__ == "__main__":
    run_backfill()
