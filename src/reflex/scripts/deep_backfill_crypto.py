import os
import sys
import argparse
import logging
import time
from datetime import datetime, timedelta
from io import BytesIO

# Third-party imports
try:
    import boto3
    import pandas as pd
    import pyarrow as pa
    import pyarrow.parquet as pq
    from dotenv import load_dotenv
except ImportError as e:
    print(f"❌ Missing dependency: {e}")
    print(
        "Please install required packages: pip install pandas pyarrow boto3 python-dotenv"
    )
    sys.exit(1)

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler()],
)
logger = logging.getLogger(__name__)

# Load environment variables
load_dotenv()


class R2Uploader:
    def __init__(self):
        self.bucket_name = os.getenv("CLOUDFLARE_BUCKET_NAME")
        self.account_id = os.getenv(
            "CLOUDFLARE_ACCESS_KEY_ID"
        )  # Usually not the account ID but let's check vars
        self.endpoint_url = os.getenv("CLOUDFLARE_STORAGE_URL")
        self.access_key = os.getenv("CLOUDFLARE_ACCESS_KEY_ID")
        self.secret_key = os.getenv("CLOUDFLARE_SECRET_ACCESS_KEY_ID")

        if not all(
            [self.bucket_name, self.endpoint_url, self.access_key, self.secret_key]
        ):
            logger.error("❌ Missing Cloudflare R2 environment variables.")
            sys.exit(1)

        self.s3 = boto3.client(
            "s3",
            endpoint_url=self.endpoint_url,
            aws_access_key_id=self.access_key,
            aws_secret_access_key=self.secret_key,
            region_name="auto",  # R2 requires a region, 'auto' is common
        )
        logger.info(f"☁️  Connected to R2 Bucket: {self.bucket_name}")

    def upload_parquet(self, key, buffer):
        try:
            buffer.seek(0)
            size_mb = buffer.getbuffer().nbytes / 1024 / 1024
            self.s3.upload_fileobj(buffer, self.bucket_name, key)
            logger.info(f"✅ Uploaded {key} ({size_mb:.2f} MB)")
        except Exception as e:
            logger.error(f"❌ Failed to upload {key}: {e}")
            raise e


class BackfillEngine:
    def __init__(self, uploader, symbol_map, dry_run=False):
        self.uploader = uploader
        self.symbol_map = symbol_map  # e.g. {'BTCUSD': 'BTC-USDT'}
        self.dry_run = dry_run

        # Buffer State
        self.current_partition_key = None
        self.dfs_buffer = []  # List of DataFrames for the current partition
        self.last_ts_ns = None

    def get_partition_key(self, ts_ns, symbol):
        # Convert nanoseconds to datetime
        dt = datetime.utcfromtimestamp(ts_ns / 1_000_000_000)
        year = dt.isocalendar()[0]
        week = dt.isocalendar()[1]
        # Format: symbol/YYYY_Www.parquet
        return f"crypto/{symbol}/{year:04}_W{week:02}.parquet"

    def process_csv(self, file_path, source_type="bitstamp"):
        logger.info(f"🚀 Starting ingestion for {file_path} [{source_type}]")

        # Define header/parsing based on source
        # Bitstamp: unix_timestamp, open, high, low, close, volume (sometimes vwap)
        # Coinbase: time, low, high, open, close, volume (Check current script)

        common_cols = ["ts", "open", "high", "low", "close", "volume"]

        if source_type == "bitstamp":
            # Found header: timestamp,open,high,low,close,volume
            read_opts = {
                "names": ["ts", "open", "high", "low", "close", "volume"],
                "header": 0,
            }
        elif source_type == "coinbase":
            # From backfill_crypto_coinbase.py: time, low, high, open, close, volume
            read_opts = {
                "names": ["ts", "low", "high", "open", "close", "volume"],
                "header": 0,
            }
        elif source_type == "gendo":
            # Unix Timestamp, Date, Symbol, Open, High, Low, Close, Volume
            read_opts = {
                "names": [
                    "ts",
                    "date",
                    "symbol",
                    "open",
                    "high",
                    "low",
                    "close",
                    "volume",
                ],
                "header": 0,
                "usecols": ["ts", "open", "high", "low", "close", "volume"],
            }
        else:
            logger.error(f"Unknown source type: {source_type}")
            return

        chunk_size = 100000
        chunks = pd.read_csv(file_path, chunksize=chunk_size, **read_opts)

        for i, chunk in enumerate(chunks):
            self.process_chunk(chunk, i)

        # Flush remaining
        if self.current_partition_key and self.dfs_buffer:
            self.flush_buffer()

        logger.info("🎉 Backfill Complete.")

    def process_chunk(self, chunk, chunk_idx):
        # 1. Normalize
        # Rename/Reorder
        if "volume_btc" in chunk.columns:
            chunk.rename(columns={"volume_btc": "volume"}, inplace=True)

        # Ensure columns exist
        req_cols = ["ts", "open", "high", "low", "close", "volume"]
        if not all(col in chunk.columns for col in req_cols):
            # Try fallback mapping or fail
            logger.warning(
                f"Chunk {chunk_idx} missing columns. Available: {chunk.columns}"
            )
            return

        df = chunk[req_cols].copy()

        # Timestamp conversion to ns
        # Handle mixed units (s, ms, us) in same chunk
        # Thresholds:
        # > 3e9: Likely ms (1970+)
        # > 3e12: Likely us (1970+) (Wait, 3e12 ms = 3e9 seconds = year 2065. So anything > 3e11 is definitely ms or us)
        # Let's use strict ranges.
        # Seconds: < 100,000,000,000 (up to year 5138)
        # MS: > 100,000,000,000 AND < 100,000,000,000,000

        # We need first to ensure it's numeric
        df["ts"] = pd.to_numeric(df["ts"])

        # Create masks
        mask_seconds = df["ts"] < 20_000_000_000  # Up to year 2603
        mask_ms = (df["ts"] >= 20_000_000_000) & (df["ts"] < 20_000_000_000_000)
        mask_us = df["ts"] >= 20_000_000_000_000

        # Apply transformations
        # Cast to float first to avoid overflow during multiplication if not careful, but we want int64 end state.
        # But if we multiply an int element by 1e9 it might overflow if it was already large.
        # But here we filter.
        # Seconds * 1e9 is fine (e.g. 1.6e9 * 1e9 = 1.6e18 < 9e18)

        df.loc[mask_seconds, "ts"] = df.loc[mask_seconds, "ts"] * 1_000_000_000
        df.loc[mask_ms, "ts"] = df.loc[mask_ms, "ts"] * 1_000_000
        df.loc[mask_us, "ts"] = df.loc[mask_us, "ts"] * 1_000

        df["ts"] = df["ts"].astype("int64")

        # Sort just in case (though we assume chronological)
        # df.sort_values('ts', inplace=True) # Resource intensive, skip if trusted source

        # Gap Detection
        self.detect_gaps(df)

        # 2. Partitioning
        # Vectorized key generation is expensive for strings.
        # But we need to group by week.
        # optimization: Convert ts to datetime, extract year/week
        temp_dt = pd.to_datetime(df["ts"], unit="ns")
        df["year"] = temp_dt.dt.isocalendar().year
        df["week"] = temp_dt.dt.isocalendar().week

        # We need a symbol. Assuming single symbol per file for now.
        # Let's take it from config or arg.
        target_symbol = self.symbol_map.get("DEFAULT", "BTC-USDT")

        # Group by Year, Week
        groups = df.groupby(["year", "week"])

        for (year, week), group in groups:
            key = f"crypto/{target_symbol}/{year:04}_W{week:02}.parquet"

            if key != self.current_partition_key:
                if self.current_partition_key is not None:
                    self.flush_buffer()
                self.current_partition_key = key

            # Drop aux columns before buffering
            clean_group = group.drop(columns=["year", "week"])
            self.dfs_buffer.append(clean_group)

    def detect_gaps(self, df):
        if self.last_ts_ns is not None:
            # Check deviation between first row and last processed
            delta = df["ts"].iloc[0] - self.last_ts_ns
            if delta > 60 * 1_000_000_000:
                minutes_missing = delta / (60 * 1_000_000_000)
                logger.warning(
                    f"⚠️  GAP DETECTED (Chunk Start): {minutes_missing:.1f} minutes missing before {df['ts'].iloc[0]}"
                )

        # Check internal gaps
        diff = df["ts"].diff()
        # Gap if diff > 60s (allow some jitter? Crypto data assumes 60s candles. strictly chronological.)
        # If strict 60s, then diff > 60s is a gap.
        gaps = diff[diff > 60 * 1_000_000_000]
        if not gaps.empty:
            # total_gap_ns = gaps.sum()  # Unused
            # Convert to minutes.
            # Actually, diff contains the time jump.
            # Time jump 120s = 1 missing candle (60s present, 60s missing... wait. 0 -> 120. Gap is 60s.)
            # Missing count = (diff / 60e9) - 1.
            missing_count = (gaps / (60 * 1_000_000_000) - 1).sum()
            logger.warning(
                f"⚠️  GAP DETECTED (Internal): Approximately {int(missing_count)} minutes missing in chunk ranges: {gaps.index.tolist()[:5]}..."
            )

        self.last_ts_ns = df["ts"].iloc[-1]

    def flush_buffer(self):
        if not self.dfs_buffer:
            return

        final_df = pd.concat(self.dfs_buffer)
        table = pa.Table.from_pandas(final_df, preserve_index=False)

        # Write to memory
        out_buffer = BytesIO()
        pq.write_table(table, out_buffer, compression="SNAPPY")

        if not self.dry_run:
            self.uploader.upload_parquet(self.current_partition_key, out_buffer)
        else:
            logger.info(
                f"[DRY RUN] Would upload {self.current_partition_key} ({out_buffer.getbuffer().nbytes} bytes)"
            )

        # Reset
        self.dfs_buffer = []


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Deep Backfill Crypto Data to R2")
    parser.add_argument("file", help="Path to CSV file")
    parser.add_argument(
        "--symbol", default="BTC-USDT", help="Target Symbol (e.g. BTC-USDT)"
    )
    parser.add_argument(
        "--source",
        choices=["bitstamp", "coinbase", "gendo"],
        default="bitstamp",
        help="Source Format",
    )
    parser.add_argument("--dry-run", action="store_true", help="Do not upload to R2")

    args = parser.parse_args()

    # Init
    if not args.dry_run:
        uploader = R2Uploader()
    else:
        uploader = None  # Engine handles None provided checks inside methods if strictly needed, but here we pass it.

        # Wait, the engine calls uploader.upload_parquet. Let's make a mock if None.
        class MockUploader:
            def upload_parquet(self, k, b):
                pass

        uploader = R2Uploader() if not args.dry_run else MockUploader()

    engine = BackfillEngine(uploader, {"DEFAULT": args.symbol}, dry_run=args.dry_run)

    if os.path.exists(args.file):
        engine.process_csv(args.file, args.source)
    else:
        print(f"File not found: {args.file}")
