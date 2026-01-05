-- Directive-86: Sovereign Interface Audit Trail
-- Stores all high-priority pilot interventions

CREATE TABLE IF NOT EXISTS sovereign_commands (
    timestamp TIMESTAMP,
    command SYMBOL,
    payload DOUBLE,
    user_id SYMBOL,
    signature STRING,
    latency_us LONG,
    source SYMBOL
) TIMESTAMP(timestamp) PARTITION BY DAY;

-- Index strategy for fast lookup
ALTER TABLE sovereign_commands ADD COLUMN IF NOT EXISTS command SYMBOL CAPACITY 256 CACHE;
ALTER TABLE sovereign_commands ADD COLUMN IF NOT EXISTS user_id SYMBOL CAPACITY 256 CACHE;
