# How-To: Run the Deep Backfill

**Problem:** You need historical data for training models.
**Solution:** Use the `deep_backfill_crypto.py` script.

## Instructions

1. **Ensure you have API Credentials** (if using live fetch) or access to the R2 bucket.

2. **Activate the Brain Environment**:

    ```bash
    source src/brain/.venv/bin/activate
    ```

3. **Run the Script**:

    ```bash
    python src/reflex/scripts/deep_backfill_crypto.py --symbol BTC-USDT --days 3650
    ```

4. **Verify Output**:
    Check `docs/internal/data/inventory.md` for the updated storage metrics.

## Troubleshooting

* **Rate Limits:** The script has auto-backoff. If it hangs, wait 5 minutes and restart; it will resume from the last checkpoint.
