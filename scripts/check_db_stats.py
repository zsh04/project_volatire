import os
import psycopg2
from dotenv import load_dotenv

# Load Env
load_dotenv()

# QuestDB PG URL
DB_URL = os.getenv("DATABASE_URL", "postgresql://admin:quest@localhost:8812/qdb")


def check_db():
    print(f"🔌 Connecting to QuestDB at {DB_URL}...")
    try:
        conn = psycopg2.connect(DB_URL)
        cur = conn.cursor()

        # 1. List Tables
        print("\n📊 Tables Found:")
        cur.execute("SELECT * FROM tables()")
        colnames = [desc[0] for desc in cur.description]
        rows = cur.fetchall()

        # Identify name column
        name_col_idx = -1
        if "name" in colnames:
            name_col_idx = colnames.index("name")
        elif "table_name" in colnames:
            name_col_idx = colnames.index("table_name")

        tables = []
        if name_col_idx != -1:
            for r in rows:
                t_name = r[name_col_idx]
                tables.append((t_name, "Unknown"))  # Partition info skipped for now
                print(f" - {t_name}")
        else:
            print("⚠️ Could not identify table name column. Columns:", colnames)

        # Update target_tables list based on discovery
        # ...

        # 2. Check Data Ranges for key tables
        target_tables = ["ohlcv_1min", "ohlcv_1d", "ohlcv_1min_backup", "market_ticks"]

        print("\n📅 Data Ranges & Partitions:")
        found_names = [t[0] for t in tables]

        for table in target_tables:
            # Check if table exists in list
            if table in found_names:
                try:
                    # Generic check for timestamp column
                    ts_col = "ts"
                    if table == "ohlcv_1d" or table == "ohlcv_1min_backup":
                        # Check columns to be sure
                        try:
                            cur.execute(f'SELECT * FROM "{table}" LIMIT 1')
                            cols = [desc[0] for desc in cur.description]
                            print(f"   Columns ({table}): {cols}")
                            for c in ["timestamp", "date", "time", "ts"]:
                                if c in cols:
                                    ts_col = c
                                    break
                        except Exception as e:
                            print(f"   Could not inspect columns for {table}: {e}")

                    if table == "ohlcv_1d":
                        ts_col = "timestamp"

                    # Check Symbols
                    cur.execute(f'SELECT distinct symbol FROM "{table}"')
                    syms = [row[0] for row in cur.fetchall()]
                    print(f" - {table} ({ts_col}):")
                    print(f"   Symbols: {syms}")

                    cur.execute(
                        f'SELECT min({ts_col}), max({ts_col}), count() FROM "{table}"'
                    )
                    stats = cur.fetchone()
                    print(f" - {table} ({ts_col}):")
                    print(f"   Start: {stats[0]}")
                    print(f"   End:   {stats[1]}")
                    print(f"   Count: {stats[2]:,}")

                    # Check partitions details
                    print(f"   Partitions (First 5 & Last 5):")
                    cur.execute(
                        f"SELECT name, minTimestamp, maxTimestamp FROM table_partitions('{table}') ORDER BY minTimestamp ASC"
                    )
                    partitions = cur.fetchall()
                    p_count = len(partitions)
                    print(f"   Total Partitions: {p_count}")

                    if p_count > 0:
                        for p in partitions[:5]:
                            print(f"     -> {p[0]} ({p[1]} to {p[2]})")
                        if p_count > 10:
                            print("     ...")
                            for p in partitions[-5:]:
                                print(f"     -> {p[0]} ({p[1]} to {p[2]})")

                except Exception as e:
                    print(f"   Error querying {table}: {e}")
            else:
                print(f" - {table}: [NOT FOUND]")

    except Exception as e:
        print(f"❌ Connection Failed: {e}")
    finally:
        if "conn" in locals():
            conn.close()


if __name__ == "__main__":
    check_db()
