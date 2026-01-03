import subprocess
import re
import sys
import time
from datetime import datetime


class SovereignAuditor:
    def __init__(self):
        self.report = []

    def log(self, message, status="INFO"):
        entry = f"[{datetime.now().isoformat()}] [{status}] {message}"
        print(entry)
        self.report.append(entry)

    def run_command(self, cmd, cwd="."):
        result = subprocess.run(
            cmd, shell=True, cwd=cwd, capture_output=True, text=True
        )
        return result

    def verify_constitutional_compliance(self):
        self.log("Verifying Constitutional Compliance (Rust Test Suite)...")
        # Run the Veto Gate tests explicitly
        cmd = "cargo test brain::veto_gate -- --nocapture"
        result = self.run_command(cmd, cwd="src/reflex")

        if result.returncode != 0:
            self.log(f"Constitutional Tests FAILED:\n{result.stderr}", "ERROR")
            return False

        # Check specific output strings to ensure the right tests ran
        if "test_nuclear_halt ... ok" in result.stdout:
            self.log("Confirmed: Nuclear Halt Logic PASSED.")
        else:
            self.log("Warning: test_nuclear_halt not found in output.", "WARN")
            return False

        if "test_anti_panic_validation ... ok" in result.stdout:
            self.log("Confirmed: Anti-Panic Logic PASSED.")
        else:
            self.log("Warning: test_anti_panic_validation not found in output.", "WARN")
            return False

        self.log("Constitutional Compliance: VERIFIED ✅", "SUCCESS")
        return True

    def verify_latency_budget(self):
        self.log("Verifying Latency Budget (Benchmarks)...")

        # We run the superposition benchmark as a proxy for 'fast path' checks
        cmd = "cargo test governor::superposition::tests::test_benchmark_speed -- --nocapture"
        result = self.run_command(cmd, cwd="src/reflex")

        latency_ns = None
        if result.returncode == 0:
            # Look for "Avg Latency: X ns"
            match = re.search(r"Avg Latency: (\d+) ns", result.stdout)
            if match:
                latency_ns = int(match.group(1))
                self.log(f"Riemann Engine Latency: {latency_ns} ns")

        if latency_ns is not None and latency_ns < 10000:
            self.log(f"Engine Latency within Budget (< 10us).", "SUCCESS")
            return True
        else:
            self.log(
                f"Latency Budget Check Failed or Parse Error. Output: {result.stdout}",
                "FAIL",
            )
            return False

    def verify_data_alignment(self):
        self.log("Verifying Data Alignment (Mock)...")
        # In a real environment, we'd query QuestDB and LanceDB.
        # Here we check for the existence of the artifacts/scripts that enforce this.

        required_files = [
            "src/reflex/scripts/fetch_vvix.py",
            "src/reflex/scripts/distilbert_processor.py",
        ]

        all_exist = True
        for f in required_files:
            check = self.run_command(
                f"ls ../../../{f}"
            )  # path relative from script exec loc if needed, but we used relative paths in list
            # Actually let's just use ls locally
            res = self.run_command(f"ls {f}", cwd=".")  # Check from root
            if res.returncode != 0:
                self.log(f"Missing Critical Data Script: {f}", "FAIL")
                all_exist = False

        if all_exist:
            self.log("Data Inestion Scripts Present.", "SUCCESS")
            return True
        return False

    def run_audit(self):
        self.log("--- STARTING SOVEREIGN AUDIT (PHASE 1-4) ---")

        checks = [
            self.verify_constitutional_compliance(),
            self.verify_latency_budget(),
            self.verify_data_alignment(),
        ]

        if all(checks):
            self.log("--- AUDIT COMPLETE: ALL SYSTEMS NOMINAL ---", "SUCCESS")
            return True
        else:
            self.log("--- AUDIT FAILED ---", "ERROR")
            return False


if __name__ == "__main__":
    auditor = SovereignAuditor()
    success = auditor.run_audit()
    if not success:
        sys.exit(1)
