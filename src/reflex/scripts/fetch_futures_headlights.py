import os
import sys
import logging
import yfinance as yf  # type: ignore
import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
import boto3
from io import BytesIO
from dotenv import load_dotenv

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("Headlights")

load_dotenv()

# R2 Configuration
CLOUDFLARE_BUCKET_NAME = os.getenv("CLOUDFLARE_BUCKET_NAME")
CLOUDFLARE_ACCESS_KEY_ID = os.getenv("CLOUDFLARE_ACCESS_KEY_ID")
CLOUDFLARE_SECRET_ACCESS_KEY_ID = os.getenv("CLOUDFLARE_SECRET_ACCESS_KEY_ID")
CLOUDFLARE_STORAGE_URL = os.getenv("CLOUDFLARE_STORAGE_URL")


class R2Uploader:
    def __init__(self, dry_run=False):
        self.bucket_name = CLOUDFLARE_BUCKET_NAME
        self.dry_run = dry_run
        if not dry_run:
            self.s3 = boto3.client(
                "s3",
                endpoint_url=CLOUDFLARE_STORAGE_URL,
                aws_access_key_id=CLOUDFLARE_ACCESS_KEY_ID,
                aws_secret_access_key=CLOUDFLARE_SECRET_ACCESS_KEY_ID,
                region_name="auto",
            )
            logger.info(f"☁️  Connected to R2 Bucket: {self.bucket_name}")
        else:
            logger.info("🚫 Dry Run Mode: S3 connection skipped.")

    def upload_parquet(self, key, buffer):
        if self.dry_run:
            buffer.seek(0)
            size_mb = buffer.getbuffer().nbytes / 1024 / 1024
            logger.info(f"🚫 [DRY RUN] Would upload {key} ({size_mb:.2f} MB)")
            return

        try:
            buffer.seek(0)
            size_mb = buffer.getbuffer().nbytes / 1024 / 1024
            self.s3.upload_fileobj(buffer, self.bucket_name, key)
            logger.info(f"✅ Uploaded {key} ({size_mb:.2f} MB)")
        except Exception as e:
            logger.error(f"❌ Failed to upload {key}: {e}")
            raise e


def fetch_and_process_headlights(dry_run=False):
    # Parameters
    futures_sym = "BTC=F"
    spot_sym = "BTC-USD"
    start_date = "2017-12-18"  # CME Futures Launch

    logger.info(f"🏎️  Fetching Institutional Headlights (CME Futures vs Spot)...")

    # 1. Fetch Data
    logger.info(f"📉 Fetching {futures_sym}...")
    fut_df = yf.Ticker(futures_sym).history(start=start_date, interval="1d")

    logger.info(f"📉 Fetching {spot_sym}...")
    spot_df = yf.Ticker(spot_sym).history(start=start_date, interval="1d")

    if fut_df.empty or spot_df.empty:
        logger.error("❌ Failed to fetch data.")
        return

    # 2. Cleanup & Processing
    # Standardize columns
    fut_df.columns = [c.lower() for c in fut_df.columns]
    spot_df.columns = [c.lower() for c in spot_df.columns]

    # NEW LOGIC: Normalize to Date (Midnight) to ensure alignment
    fut_df.index = pd.to_datetime(fut_df.index.date)
    spot_df.index = pd.to_datetime(spot_df.index.date)

    # Prepare for merge
    fut_ready = fut_df[["open", "high", "low", "close"]].copy()
    fut_ready.columns = ["open_fut", "high_fut", "low_fut", "close_fut"]

    spot_ready = spot_df[["close"]].copy()
    spot_ready.columns = ["close_spot"]

    # Merge Inner Join on Index (Date)
    merged = fut_ready.merge(spot_ready, left_index=True, right_index=True, how="inner")

    merged["timestamp"] = merged.index

    # 3. Calculate Logic
    # Basis = (Fut - Spot)/Spot * (365/30)
    # Assumes ~30 days to expiry for the continuous front month
    DAYS_TO_EXPIRY_PROXY = 30
    merged["basis_ua"] = (merged["close_fut"] - merged["close_spot"]) / merged[
        "close_spot"
    ]
    merged["basis_annualized"] = merged["basis_ua"] * (365 / DAYS_TO_EXPIRY_PROXY)

    # Gap Detection
    # Gap = Today's Open - Yesterday's Close
    merged["prev_close_fut"] = merged["close_fut"].shift(1)
    merged["gap_val"] = merged["open_fut"] - merged["prev_close_fut"]
    merged["gap_pct"] = merged["gap_val"] / merged["prev_close_fut"]

    # Filter columns for export
    final_cols = [
        "timestamp",
        "open_fut",
        "high_fut",
        "low_fut",
        "close_fut",
        "close_spot",
        "basis_annualized",
        "gap_pct",
    ]

    df_export = merged[final_cols].copy()

    # Clean NaN
    df_export = df_export.dropna()

    logger.info(f"✅ Processed {len(df_export)} aligned rows.")

    # 4. Upload
    key = "crypto/futures/BTC1!/btc1_daily_2017_2025.parquet"

    table = pa.Table.from_pandas(df_export)
    buffer = BytesIO()
    pq.write_table(table, buffer, compression="snappy")

    logger.info(f"🚀 Uploading to: {key}")
    uploader = R2Uploader(dry_run=dry_run)
    uploader.upload_parquet(key, buffer)


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    try:
        fetch_and_process_headlights(dry_run=args.dry_run)
        logger.info("🎉 Headlights verification complete.")
    except Exception as e:
        logger.error(f"💥 Failed: {e}")
        sys.exit(1)
