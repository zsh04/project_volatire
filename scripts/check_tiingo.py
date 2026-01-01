import requests
import os
from dotenv import load_dotenv

load_dotenv()

TIINGO_KEY = os.getenv("TIINGO_API_KEY")
URL = "https://api.tiingo.com/tiingo/crypto/prices"


def check_tiingo(symbol, date):
    params = {
        "tickers": symbol,
        "startDate": date,
        "resampleFreq": "1min",
        "token": TIINGO_KEY,
    }
    resp = requests.get(URL, params=params)
    print(f"Status: {resp.status_code}")
    print(f"Data: {resp.json()[:3] if resp.status_code == 200 else resp.text}")


if __name__ == "__main__":
    for year in range(2015, 2020):
        date = f"{year}-01-02"
        print(f"Checking {date}...")
        check_tiingo("btcusd", date)
