import os
import boto3
import logging
from dotenv import load_dotenv

load_dotenv()

# Setup Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("R2Audit")

CLOUDFLARE_BUCKET_NAME = os.getenv("CLOUDFLARE_BUCKET_NAME")
CLOUDFLARE_ACCESS_KEY_ID = os.getenv("CLOUDFLARE_ACCESS_KEY_ID")
CLOUDFLARE_SECRET_ACCESS_KEY_ID = os.getenv("CLOUDFLARE_SECRET_ACCESS_KEY_ID")
CLOUDFLARE_STORAGE_URL = os.getenv("CLOUDFLARE_STORAGE_URL")


def get_prefix_size(prefix):
    s3 = boto3.client(
        "s3",
        endpoint_url=CLOUDFLARE_STORAGE_URL,
        aws_access_key_id=CLOUDFLARE_ACCESS_KEY_ID,
        aws_secret_access_key=CLOUDFLARE_SECRET_ACCESS_KEY_ID,
        region_name="auto",
    )

    total_size = 0
    paginator = s3.get_paginator("list_objects_v2")

    # Ensure prefix ends with / if it's a directory
    if not prefix.endswith("/") and prefix != "":
        prefix += "/"

    logger.info(f"Calculating size for s3://{CLOUDFLARE_BUCKET_NAME}/{prefix} ...")

    for page in paginator.paginate(Bucket=CLOUDFLARE_BUCKET_NAME, Prefix=prefix):
        if "Contents" in page:
            for obj in page["Contents"]:
                total_size += obj["Size"]

    return total_size


def fmt_size(num):
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if abs(num) < 1024.0:
            return "%3.2f %s" % (num, unit)
        num /= 1024.0
    return "%.2f %s" % (num, "PB")


if __name__ == "__main__":
    prefixes = ["crypto", "equities", "equities_daily", "equities_daily/VVIX"]
    for p in prefixes:
        try:
            size = get_prefix_size(p)
            print(f"Directory: {p} | Size: {fmt_size(size)}")
        except Exception as e:
            print(f"Error checking {p}: {e}")
