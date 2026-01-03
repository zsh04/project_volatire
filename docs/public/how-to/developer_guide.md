# Developer Guide

**Public Guide**

## 1. Getting Started

### Clone & Build

```bash
git clone https://github.com/project-voltaire/voltaire.git
cd voltaire

# Build Rust Engines
cd src/reflex
cargo build
```

## 2. Testing Strategy

We employ a "Sovereign Audit" testing strategy:

1. **Unit Tests**: Fast, localized logic checks.

    ```bash
    cargo test
    ```

2. **Integration Tests**: "Golden Thread" tests verifying the full OODA loop.

    ```bash
    cargo test --test verify_telemetry
    ```

3. **Verification Scripts**: Python scripts in `src/reflex/scripts/` for complex validation (e.g., Stress Tests).

## 3. Adding Dependencies

* **Rust**: Edit `src/reflex/Cargo.toml`. Prefer `deadpool` for pooling and `tokio` for async.
* **Python**: Manage via `uv` or `pip`. Update `requirements.txt`.

## 4. Coding Standards (The Code Constitution)

* **Safety First**: No `unwrap()` in production paths. Use `?` or `map_err`.
* **Telemetry**: Every major loop must emit a span (`#[tracing::instrument]`).
* **Strict Types**: Use New Types (e.g., `OrderId(String)`) over primitives where possible.

## 5. Debugging

* **Logs**: Check `docker logs cc_pulse` for telemetry issues.
* **Traces**: Use Grafana Cloud (if configured) or local Jaeger to visualize `orient` -> `decide` latencies.
