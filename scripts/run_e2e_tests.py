#!/usr/bin/env python3
"""
Nexa Skill Compiler - E2E Test Runner

This script runs end-to-end tests on the NSC compiler,
testing the full compilation pipeline with real SKILL.md files.
"""

import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class TestResult:
    """Test result data"""
    name: str
    success: bool
    duration_ms: float
    error_message: Optional[str] = None
    output_files: list[str] = None

    def __post_init__(self):
        if self.output_files is None:
            self.output_files = []


class E2ETestRunner:
    """E2E test runner for Nexa Skill Compiler"""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.fixtures_dir = project_root / "tests" / "fixtures"
        self.output_dir = project_root / "target" / "e2e-output"
        self.binary_path = project_root / "target" / "release" / "nexa-skill"
        self.results: list[TestResult] = []

    def setup(self) -> bool:
        """Setup test environment"""
        print("🔧 Setting up E2E test environment...")

        # Create output directory
        self.output_dir.mkdir(parents=True, exist_ok=True)

        # Check binary exists
        if not self.binary_path.exists():
            print(f"❌ Binary not found: {self.binary_path}")
            print("   Run 'cargo build --release' first")
            return False

        # Check fixtures exist
        if not self.fixtures_dir.exists():
            print(f"❌ Fixtures directory not found: {self.fixtures_dir}")
            return False

        print("✅ Setup complete")
        return True

    def run_single_test(self, skill_file: Path) -> TestResult:
        """Run a single E2E test"""
        test_name = skill_file.stem
        start_time = time.time()

        # Create output directory for this test
        test_output_dir = self.output_dir / test_name
        test_output_dir.mkdir(exist_ok=True)

        # Run compilation
        cmd = [
            str(self.binary_path),
            "build",
            str(skill_file),  # INPUT is positional argument
            "--out-dir", str(test_output_dir),
            "--claude",
            "--codex",
            "--gemini",
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30,
                cwd=str(self.project_root)
            )

            duration_ms = (time.time() - start_time) * 1000

            if result.returncode != 0:
                return TestResult(
                    name=test_name,
                    success=False,
                    duration_ms=duration_ms,
                    error_message=result.stderr or result.stdout
                )

            # Check output files
            output_files = list(test_output_dir.glob("**/*"))
            output_file_names = [str(f.relative_to(test_output_dir)) for f in output_files if f.is_file()]

            return TestResult(
                name=test_name,
                success=True,
                duration_ms=duration_ms,
                output_files=output_file_names
            )

        except subprocess.TimeoutExpired:
            duration_ms = (time.time() - start_time) * 1000
            return TestResult(
                name=test_name,
                success=False,
                duration_ms=duration_ms,
                error_message="Test timed out after 30 seconds"
            )
        except Exception as e:
            duration_ms = (time.time() - start_time) * 1000
            return TestResult(
                name=test_name,
                success=False,
                duration_ms=duration_ms,
                error_message=str(e)
            )

    def run_all_tests(self) -> bool:
        """Run all E2E tests"""
        print("\n🚀 Running E2E tests...\n")

        # Get all skill files
        skill_files = list(self.fixtures_dir.glob("*.md"))

        if not skill_files:
            print("❌ No test files found")
            return False

        print(f"Found {len(skill_files)} test files:\n")
        for f in skill_files:
            print(f"  - {f.name}")

        print("\n" + "=" * 60 + "\n")

        # Run each test
        for skill_file in skill_files:
            print(f"▶️  Testing: {skill_file.name}")
            result = self.run_single_test(skill_file)
            self.results.append(result)

            if result.success:
                print(f"   ✅ PASSED ({result.duration_ms:.1f}ms)")
                if result.output_files:
                    print(f"   📁 Output files: {', '.join(result.output_files)}")
            else:
                print(f"   ❌ FAILED ({result.duration_ms:.1f}ms)")
                if result.error_message:
                    print(f"   ⚠️  Error: {result.error_message[:200]}")
            print()

        return self._print_summary()

    def _print_summary(self) -> bool:
        """Print test summary"""
        total = len(self.results)
        passed = sum(1 for r in self.results if r.success)
        failed = total - passed

        print("=" * 60)
        print("\n📊 Test Summary\n")
        print(f"  Total:  {total}")
        print(f"  ✅ Passed: {passed}")
        print(f"  ❌ Failed: {failed}")

        if failed > 0:
            print("\n❌ Failed tests:")
            for r in self.results:
                if not r.success:
                    print(f"  - {r.name}")

        # Calculate average duration
        avg_duration = sum(r.duration_ms for r in self.results) / total if total > 0 else 0
        print(f"\n⏱️  Average duration: {avg_duration:.1f}ms")

        # Save results to JSON
        results_file = self.output_dir / "test-results.json"
        results_data = {
            "total": total,
            "passed": passed,
            "failed": failed,
            "average_duration_ms": avg_duration,
            "tests": [
                {
                    "name": r.name,
                    "success": r.success,
                    "duration_ms": r.duration_ms,
                    "error_message": r.error_message,
                    "output_files": r.output_files
                }
                for r in self.results
            ]
        }

        with open(results_file, "w") as f:
            json.dump(results_data, f, indent=2)

        print(f"\n📄 Results saved to: {results_file}")

        return failed == 0


def main():
    """Main entry point"""
    project_root = Path(__file__).parent.parent

    runner = E2ETestRunner(project_root)

    if not runner.setup():
        sys.exit(1)

    success = runner.run_all_tests()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()