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
logger = logging.getLogger("VVIXFetcher")

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


def fetch_and_upload_vvix(dry_run=False):
    symbol = "^VVIX"
    logger.info(f"📉 Fetching 20y history for {symbol} via yfinance...")

    # 1. Fetch Data
    ticker = yf.Ticker(symbol)
    df = ticker.history(period="max")

    if df.empty:
        logger.error(f"❌ No data found for {symbol}")
        return

    logger.info(
        f"✅ Fetched {len(df)} rows. Range: {df.index.min()} to {df.index.max()}"
    )

    # 2. Process Data
    # Ensure index is timezone naive or UTC for consistency
    if df.index.tz is not None:
        df.index = df.index.tz_convert(None)

    # Reset index to make Date a column
    df = df.reset_index()

    # Standardize column names to lowercase
    df.columns = [c.lower() for c in df.columns]

    # 3. Convert to Parquet
    table = pa.Table.from_pandas(df)
    buffer = BytesIO()
    pq.write_table(table, buffer, compression="snappy")

    # 4. Upload to R2
    file_key = "equities_daily/VVIX/history.parquet"
    uploader = R2Uploader(dry_run=dry_run)
    uploader.upload_parquet(file_key, buffer)


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Fetch VVIX and upload to R2")
    parser.add_argument(
        "--dry-run", action="store_true", help="Perform a dry run without uploading"
    )
    args = parser.parse_args()

    try:
        fetch_and_upload_vvix(dry_run=args.dry_run)
        logger.info("🎉 Helper script completed successfully.")
    except Exception as e:
        logger.error(f"💥 Script failed: {e}")
        sys.exit(1)
