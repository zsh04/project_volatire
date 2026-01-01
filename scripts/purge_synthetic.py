import socket
import datetime

QUESTDB_HOST = "localhost"
QUESTDB_PORT = 8812  # PG Wire Port
TABLE = "ohlcv_1min"


def drop_synthetic_partitions():
    # We generated data from 2015-01-01 to 2019-01-01.
    # Partitions are likely WEEKLY based on previous logs (2019-W39).
    # We will generate DROP PARTITION commands for this range.

    import psycopg2

    try:
        conn = psycopg2.connect(
            host=QUESTDB_HOST,
            port=QUESTDB_PORT,
            user="admin",
            password="quest",
            database="qdb",
        )
        conn.autocommit = True
        cur = conn.cursor()

        print(f"🔌 Connected to QuestDB. Scanning partitions for {TABLE}...")

        # Get list of partitions to be safe
        cur.execute(
            f"SELECT name, minTimestamp, maxTimestamp FROM table_partitions('{TABLE}')"
        )
        partitions = cur.fetchall()

        print(f"📊 Found {len(partitions)} partitions.")

        start_cut = datetime.datetime(2015, 1, 1)
        end_cut = datetime.datetime(2019, 1, 2)

        for p in partitions:
            p_name = p[0]
            # QuestDB partition format usually YYYY-MM or YYYY-Www or YYYY-MM-DD
            # We can use the minTimestamp to decide
            p_min_ts = p[1]
            # Handle timezone awareness
            if p_min_ts.tzinfo is not None:
                p_min_ts = p_min_ts.replace(tzinfo=None)

            if p_min_ts >= start_cut and p_min_ts < end_cut:
                print(f"🔥 Dropping Synthetic Partition: {p_name} ({p_min_ts})")
                try:
                    cur.execute(f"ALTER TABLE {TABLE} DROP PARTITION LIST '{p_name}'")
                except Exception as e:
                    print(f"⚠️ Failed to drop {p_name}: {e}")

        print("✅ Purge Complete.")

    except Exception as e:
        print(f"❌ Database Error: {e}")


if __name__ == "__main__":
    drop_synthetic_partitions()
