#!/usr/bin/env python3
"""
Nexa Skill Compiler - Large Scale E2E Test Runner
Runs comprehensive tests on 60+ skill files
"""

import json
import os
import subprocess
import sys
import time
import argparse
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, List
from datetime import datetime


@dataclass
class TestResult:
    """Test result data"""
    name: str
    success: bool
    duration_ms: float
    error_message: Optional[str] = None
    output_files: List[str] = field(default_factory=list)
    category: str = "unknown"


@dataclass
class TestStatistics:
    """Test statistics"""
    total: int = 0
    passed: int = 0
    failed: int = 0
    total_duration_ms: float = 0.0
    by_category: dict = field(default_factory=dict)
    
    def add_result(self, result: TestResult):
        self.total += 1
        self.total_duration_ms += result.duration_ms
        if result.success:
            self.passed += 1
        else:
            self.failed += 1
        
        if result.category not in self.by_category:
            self.by_category[result.category] = {"passed": 0, "failed": 0}
        
        if result.success:
            self.by_category[result.category]["passed"] += 1
        else:
            self.by_category[result.category]["failed"] += 1


class LargeScaleTestRunner:
    """Large scale E2E test runner for Nexa Skill Compiler"""

    def __init__(self, project_root: Path, fixtures_dir: Path = None):
        self.project_root = project_root
        self.fixtures_dir = fixtures_dir or project_root / "tests" / "fixtures"
        self.output_dir = project_root / "target" / "large-scale-output"
        self.binary_path = project_root / "target" / "release" / "nexa-skill"
        self.results: List[TestResult] = []
        self.stats = TestStatistics()

    def setup(self) -> bool:
        """Setup test environment"""
        print("🔧 Setting up large scale test environment...")

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

    def get_category(self, filename: str) -> str:
        """Determine test category from filename"""
        if filename.startswith("error-"):
            return "error_handling"
        elif "simple" in filename:
            return "simple"
        elif "medium" in filename:
            return "medium"
        elif "complex" in filename:
            return "complex"
        elif "minimal" in filename:
            return "minimal"
        else:
            return "standard"

    def run_single_test(self, skill_file: Path) -> TestResult:
        """Run a single E2E test"""
        test_name = skill_file.stem
        category = self.get_category(test_name)
        start_time = time.time()

        # Create output directory for this test
        test_output_dir = self.output_dir / test_name
        test_output_dir.mkdir(exist_ok=True)

        # Run compilation
        cmd = [
            str(self.binary_path),
            "build",
            str(skill_file),
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
                    error_message=result.stderr or result.stdout,
                    category=category
                )

            # Collect output files
            output_files = []
            if test_output_dir.exists():
                output_files = [f.name for f in test_output_dir.iterdir() if f.is_file()]

            return TestResult(
                name=test_name,
                success=True,
                duration_ms=duration_ms,
                output_files=output_files,
                category=category
            )

        except subprocess.TimeoutExpired:
            duration_ms = (time.time() - start_time) * 1000
            return TestResult(
                name=test_name,
                success=False,
                duration_ms=duration_ms,
                error_message="Test timed out after 30 seconds",
                category=category
            )
        except Exception as e:
            duration_ms = (time.time() - start_time) * 1000
            return TestResult(
                name=test_name,
                success=False,
                duration_ms=duration_ms,
                error_message=str(e),
                category=category
            )

    def run_all_tests(self) -> TestStatistics:
        """Run all E2E tests"""
        # Find all .md files
        skill_files = sorted(self.fixtures_dir.glob("*.md"))
        
        if not skill_files:
            print(f"❌ No .md files found in {self.fixtures_dir}")
            return self.stats

        print(f"\n🚀 Running large scale E2E tests...")
        print(f"\nFound {len(skill_files)} test files:\n")

        # Group by category for display
        categories = {}
        for f in skill_files:
            cat = self.get_category(f.stem)
            if cat not in categories:
                categories[cat] = []
            categories[cat].append(f)

        for cat, files in categories.items():
            print(f"  {cat}: {len(files)} files")
        print()

        # Run tests
        for i, skill_file in enumerate(skill_files, 1):
            result = self.run_single_test(skill_file)
            self.results.append(result)
            self.stats.add_result(result)

            # Print progress
            status = "✅ PASSED" if result.success else "❌ FAILED"
            print(f"[{i:3d}/{len(skill_files)}] {status} ({result.duration_ms:.1f}ms) - {result.name}")

        return self.stats

    def print_summary(self):
        """Print test summary"""
        print("\n" + "=" * 60)
        print("📊 Large Scale Test Summary")
        print("=" * 60)
        print(f"\n  Total:   {self.stats.total}")
        print(f"  ✅ Passed: {self.stats.passed}")
        print(f"  ❌ Failed: {self.stats.failed}")
        print(f"\n  Pass Rate: {self.stats.passed/self.stats.total*100:.1f}%")
        print(f"  ⏱️  Total time: {self.stats.total_duration_ms/1000:.2f}s")
        print(f"  ⏱️  Average: {self.stats.total_duration_ms/self.stats.total:.2f}ms")

        print("\n📈 Results by Category:")
        for cat, stats in self.stats.by_category.items():
            total = stats["passed"] + stats["failed"]
            rate = stats["passed"] / total * 100 if total > 0 else 0
            print(f"  {cat:20s}: {stats['passed']:3d}/{total:3d} ({rate:5.1f}%)")

        # List failed tests
        failed_tests = [r for r in self.results if not r.success]
        if failed_tests:
            print(f"\n❌ Failed tests ({len(failed_tests)}):")
            for r in failed_tests[:10]:  # Show first 10
                print(f"  - {r.name}")
            if len(failed_tests) > 10:
                print(f"  ... and {len(failed_tests) - 10} more")

    def save_results(self):
        """Save results to JSON"""
        results_data = {
            "timestamp": datetime.now().isoformat(),
            "summary": {
                "total": self.stats.total,
                "passed": self.stats.passed,
                "failed": self.stats.failed,
                "pass_rate": self.stats.passed / self.stats.total * 100 if self.stats.total > 0 else 0,
                "total_duration_ms": self.stats.total_duration_ms,
                "average_duration_ms": self.stats.total_duration_ms / self.stats.total if self.stats.total > 0 else 0,
            },
            "by_category": self.stats.by_category,
            "results": [
                {
                    "name": r.name,
                    "success": r.success,
                    "duration_ms": r.duration_ms,
                    "category": r.category,
                    "error_message": r.error_message,
                    "output_files": r.output_files,
                }
                for r in self.results
            ]
        }

        results_path = self.output_dir / "test-results.json"
        results_path.write_text(json.dumps(results_data, indent=2))
        print(f"\n📄 Results saved to: {results_path}")


def main():
    parser = argparse.ArgumentParser(description="Large Scale E2E Test Runner")
    parser.add_argument(
        "fixtures_dir",
        nargs="?",
        help="Directory containing test fixtures",
    )
    args = parser.parse_args()

    project_root = Path(__file__).parent.parent
    fixtures_dir = Path(args.fixtures_dir) if args.fixtures_dir else None

    runner = LargeScaleTestRunner(project_root, fixtures_dir)

    if not runner.setup():
        sys.exit(1)

    runner.run_all_tests()
    runner.print_summary()
    runner.save_results()

    # Exit with error code if any tests failed
    sys.exit(0 if runner.stats.failed == 0 else 1)


if __name__ == "__main__":
    main()