import os
import sys
import logging
import psycopg2
import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
import boto3
import argparse
from io import BytesIO
from datetime import datetime, timedelta
from dotenv import load_dotenv

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("R2Migration")

load_dotenv()

# Configuration
QUESTDB_HOST = "localhost"
QUESTDB_PORT = 8812  # Postgres wire protocol default for QuestDB
QUESTDB_USER = "admin"
QUESTDB_PASSWORD = "quest"
QUESTDB_DB = "qdb"

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


class MigrationEngine:
    def __init__(self, dry_run=False):
        self.uploader = R2Uploader(dry_run=dry_run)
        self.conn = self._connect_db()

    def _connect_db(self):
        try:
            conn = psycopg2.connect(
                host=QUESTDB_HOST,
                port=QUESTDB_PORT,
                user=QUESTDB_USER,
                password=QUESTDB_PASSWORD,
                dbname=QUESTDB_DB,
            )
            logger.info(f"🔌 Connected to QuestDB at {QUESTDB_HOST}:{QUESTDB_PORT}")
            return conn
        except Exception as e:
            logger.error(f"❌ Failed to connect to QuestDB: {e}")
            sys.exit(1)

    def get_symbols(self, table):
        query = f"SELECT distinct symbol FROM {table} ORDER BY symbol"
        df = pd.read_sql(query, self.conn)
        return df["symbol"].tolist()

    def _migrate_symbol(self, table_name, symbol, prefix, partition_freq):
        # Determine time range for the symbol
        min_max_query = f"SELECT min(timestamp), max(timestamp) FROM {table_name} WHERE symbol = '{symbol}'"
        cursor = self.conn.cursor()
        cursor.execute(min_max_query)
        min_ts, max_ts = cursor.fetchone()

        if not min_ts or not max_ts:
            logger.warning(f"⚠️  No data for {symbol}, skipping.")
            return

        logger.info(f"Migrating {symbol} | Range: {min_ts} -> {max_ts}")

        current_start = min_ts

        # Let's align current_start to start of year
        current_year = current_start.year
        end_year = max_ts.year

        for year in range(current_year, end_year + 1):
            start_date = datetime(year, 1, 1)
            end_date = datetime(year + 1, 1, 1)

            # QuestDB SQL filter
            query = f"""
                SELECT * FROM {table_name} 
                WHERE symbol = '{symbol}' 
                AND timestamp >= '{start_date.strftime("%Y-%m-%d")}' 
                AND timestamp < '{end_date.strftime("%Y-%m-%d")}'
            """

            try:
                df = pd.read_sql(query, self.conn)
            except Exception as e:
                logger.error(f"Error fetching {symbol} {year}: {e}")
                continue

            if df.empty:
                continue

            # Rename 'timestamp' to 'ts' for consistency
            if "timestamp" in df.columns:
                df.rename(columns={"timestamp": "ts"}, inplace=True)

            # Ensure ts is int64 (nanoseconds)
            if pd.api.types.is_datetime64_any_dtype(df["ts"]):
                df["ts"] = df["ts"].astype("int64")  # nanoseconds

            # Process partitions
            self._upload_partitions(df, prefix, symbol, partition_freq)

    def _upload_partitions(self, df, prefix, symbol, partition_freq):
        # Convert ns timestamp back to datetime for grouping
        df["dt"] = pd.to_datetime(df["ts"])

        if partition_freq == "WEEK":
            # Group by Year and Week
            groups = df.groupby(
                [df["dt"].dt.isocalendar().year, df["dt"].dt.isocalendar().week]
            )

            for (year, week), group in groups:
                partition_key = f"{prefix}/{symbol}/{year}_W{week:02d}.parquet"
                self._write_and_upload(group, partition_key)

        elif partition_freq == "YEAR":
            # Group by Year
            groups = df.groupby(df["dt"].dt.year)

            for year, group in groups:
                partition_key = f"{prefix}/{symbol}/{year}.parquet"
                self._write_and_upload(group, partition_key)

    def _write_and_upload(self, df, key):
        # Drop the temp 'dt' column
        cols_to_save = [c for c in df.columns if c != "dt"]
        final_df = df[cols_to_save]

        # Write to Buffer
        out_buffer = BytesIO()
        final_df.to_parquet(out_buffer, index=False, compression="snappy")

        # Upload
        self.uploader.upload_parquet(key, out_buffer)


if __name__ == "__main__":
    if not CLOUDFLARE_BUCKET_NAME:
        logger.error("Missing Environment Variables")
        sys.exit(1)

    parser = argparse.ArgumentParser(description="Migrate QuestDB to R2")
    parser.add_argument("--symbol", help="Limit to specific symbol", default=None)
    parser.add_argument(
        "--resume-from", help="Resume migration from this symbol onwards", default=None
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Don't upload, just process"
    )

    args = parser.parse_args()

    migrator = MigrationEngine(dry_run=args.dry_run)

    if args.dry_run:
        logger.info("🚫 DRY RUN MODE: No data will be uploaded.")

    # 1. Migrate Intraday (10 years)
    logger.info("=== Migrating Intraday Equities ===")

    # Filter symbols if arg provided
    target_symbols = migrator.get_symbols("ohlcv_1min_backup")

    if args.symbol:
        if args.symbol in target_symbols:
            target_symbols = [args.symbol]
        else:
            if args.symbol not in migrator.get_symbols("ohlcv_1d"):
                logger.warning(f"Symbol {args.symbol} not found in ohlcv_1min_backup")
            target_symbols = []

    if args.resume_from:
        try:
            # Find index and slice
            start_idx = target_symbols.index(args.resume_from)
            logger.info(
                f"⏩ Resuming from {args.resume_from} (Index {start_idx}/{len(target_symbols)})"
            )
            target_symbols = target_symbols[start_idx:]
        except ValueError:
            logger.warning(
                f"Resume symbol {args.resume_from} not found in list. Starting from beginning."
            )

    for symbol in target_symbols:
        migrator._migrate_symbol("ohlcv_1min_backup", symbol, "equities", "WEEK")

    # 2. Migrate Daily (20 years)
    logger.info("=== Migrating Daily Equities ===")
    target_symbols_daily = migrator.get_symbols("ohlcv_1d")
    if args.symbol:
        if args.symbol in target_symbols_daily:
            target_symbols_daily = [args.symbol]
        else:
            target_symbols_daily = []

    if args.resume_from:
        try:
            # Find index and slice
            start_idx = target_symbols_daily.index(args.resume_from)
            target_symbols_daily = target_symbols_daily[start_idx:]
        except ValueError:
            pass  # Might calculate index on intraday list but not present in daily, proceed normally or ignore.

    for symbol in target_symbols_daily:
        migrator._migrate_symbol("ohlcv_1d", symbol, "equities_daily", "YEAR")

    logger.info("🎉 Migration Complete")
