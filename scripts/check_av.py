import requests
import os
from dotenv import load_dotenv

load_dotenv()

AV_KEY = os.getenv("ALPHAVANTAGE_API_KEY")
URL = "https://www.alphavantage.co/query"


def check_av(symbol):
    params = {
        "function": "CRYPTO_INTRADAY",
        "symbol": symbol,
        "market": "USD",
        "interval": "1min",
        "outputsize": "full",
        "apikey": AV_KEY,
    }
    print(f"checking AV for {symbol}...")
    resp = requests.get(URL, params=params)
    data = resp.json()

    meta = data.get("Meta Data", {})
    ts_data = data.get("Time Series Crypto (1min)", {})

    print(f"Status: {resp.status_code}")
    if ts_data:
        timestamps = sorted(ts_data.keys())
        print(f"First Candle: {timestamps[0]}")
        print(f"Last Candle: {timestamps[-1]}")
        print(f"Total Count: {len(timestamps)}")
    else:
        print(f"No Data or Error: {data}")


if __name__ == "__main__":
    check_av("BTC")
