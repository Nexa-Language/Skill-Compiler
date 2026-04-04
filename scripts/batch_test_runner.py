#!/usr/bin/env python3
"""
Batch Test Runner for Nexa Skill Compiler

This script runs E2E tests in batches, using real skill datasets fetched
from GitHub repositories. It supports progressive testing from simple to
complex skills, with detailed result collection and reporting.

Usage:
    python scripts/batch_test_runner.py --batch 1
    python scripts/batch_test_runner.py --all
    python scripts/batch_test_runner.py --batch 2 --limit 5
"""

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict, field
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional, Any
from enum import Enum


# ============================================================================
# Configuration
# ============================================================================

PROJECT_ROOT = Path(__file__).parent.parent
FIXTURES_DIR = PROJECT_ROOT / "tests" / "fixtures" / "real_corpus"
RESULTS_DIR = PROJECT_ROOT / "test_results"
REPORTS_DIR = RESULTS_DIR / "reports"
MANIFEST_FILE = FIXTURES_DIR / "manifest.json"

# Default test configuration
DEFAULT_BATCH_SIZE = 10
DEFAULT_TEST_INTERVAL = 2  # seconds between tests
DEFAULT_TIMEOUT = 60  # seconds per test


class TestStatus(Enum):
    """Test execution status."""
    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"
    ERROR = "error"


class ComplexityLevel(Enum):
    """Skill complexity level."""
    SIMPLE = "simple"
    MEDIUM = "medium"
    COMPLEX = "complex"


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class TestResult:
    """Result of a single test execution."""
    test_id: str
    skill_name: str
    batch_id: int
    complexity: str
    status: str  # Use string instead of enum for JSON serialization
    duration_ms: float
    targets_tested: List[str]
    targets_passed: List[str]
    targets_failed: List[str]
    diagnostics: List[Dict[str, Any]]
    error_message: Optional[str] = None
    output_files: List[str] = field(default_factory=list)
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class BatchResult:
    """Result of a batch test execution."""
    batch_id: int
    batch_name: str
    total_tests: int
    passed: int
    failed: int
    skipped: int
    errors: int
    pass_rate: float
    avg_duration_ms: float
    total_duration_s: float
    results: List[TestResult]
    started_at: str
    completed_at: str


@dataclass
class TestReport:
    """Complete test report with all batch results."""
    version: str
    generated_at: str
    total_batches: int
    total_tests: int
    total_passed: int
    total_failed: int
    overall_pass_rate: float
    batch_results: List[Dict[str, Any]]
    summary_by_complexity: Dict[str, Dict[str, int]]
    summary_by_source: Dict[str, Dict[str, int]]
    performance_metrics: Dict[str, float]


# ============================================================================
# Helper Functions
# ============================================================================

def log_info(msg: str) -> None:
    """Print info message with timestamp."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"[{timestamp}] [INFO] {msg}")


def log_warn(msg: str) -> None:
    """Print warning message."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"[{timestamp}] [WARN] {msg}")


def log_error(msg: str) -> None:
    """Print error message."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"[{timestamp}] [ERROR] {msg}")


def log_success(msg: str) -> None:
    """Print success message."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"[{timestamp}] [✓] {msg}")


def log_fail(msg: str) -> None:
    """Print failure message."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"[{timestamp}] [✗] {msg}")


def run_command(
    cmd: List[str],
    cwd: Optional[Path] = None,
    timeout: int = DEFAULT_TIMEOUT
) -> subprocess.CompletedProcess:
    """Run a shell command with timeout."""
    try:
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(
            args=cmd,
            returncode=-1,
            stdout="",
            stderr="Command timed out",
        )


def load_manifest() -> Optional[Dict[str, Any]]:
    """Load the skill manifest file."""
    if not MANIFEST_FILE.exists():
        log_warn(f"Manifest file not found: {MANIFEST_FILE}")
        return None
    
    with open(MANIFEST_FILE, "r", encoding="utf-8") as f:
        return json.load(f)


def get_skills_for_batch(batch_id: int, manifest: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Get skills for a specific batch from manifest."""
    batch_key = str(batch_id)
    if batch_key not in manifest.get("batches", {}):
        return []
    
    batch_skill_names = manifest["batches"][batch_key]
    skills = []
    
    for skill in manifest.get("skills", []):
        if skill["name"] in batch_skill_names:
            skills.append(skill)
    
    return skills


def get_skill_file(skill_name: str) -> Optional[Path]:
    """Get the skill file path for a given skill name."""
    skill_file = FIXTURES_DIR / f"{skill_name}.md"
    if skill_file.exists():
        return skill_file
    
    # Try alternative naming
    for f in FIXTURES_DIR.glob("*.md"):
        if skill_name.lower() in f.stem.lower():
            return f
    
    return None


# ============================================================================
# Test Execution
# ============================================================================

def compile_skill(skill_file: Path, targets: List[str] = None) -> subprocess.CompletedProcess:
    """Compile a skill file using the NSC CLI."""
    if targets is None:
        targets = ["claude", "codex", "gemini"]
    
    # Use release binary directly for faster execution
    binary_path = PROJECT_ROOT / "target" / "release" / "nexa-skill"
    
    if binary_path.exists():
        cmd = [
            str(binary_path),
            "build",
            str(skill_file),
            "--all",
            "--out-dir",
            str(RESULTS_DIR / "compiled"),
        ]
    else:
        # Fallback to cargo run
        cmd = [
            "cargo",
            "run",
            "--release",
            "--bin",
            "nexa-skill",
            "--",
            "build",
            str(skill_file),
            "--all",
            "--out-dir",
            str(RESULTS_DIR / "compiled"),
        ]
    
    return run_command(cmd, timeout=DEFAULT_TIMEOUT)


def check_skill(skill_file: Path) -> subprocess.CompletedProcess:
    """Check a skill file for diagnostics."""
    binary_path = PROJECT_ROOT / "target" / "release" / "nexa-skill"
    
    if binary_path.exists():
        cmd = [
            str(binary_path),
            "check",
            str(skill_file),
        ]
    else:
        cmd = [
            "cargo",
            "run",
            "--release",
            "--bin",
            "nexa-skill",
            "--",
            "check",
            str(skill_file),
        ]
    
    return run_command(cmd, timeout=30)


def parse_diagnostics(output: str) -> List[Dict[str, Any]]:
    """Parse diagnostic information from CLI output."""
    diagnostics = []
    
    # Simple parsing - look for error/warning patterns
    lines = output.split("\n")
    for line in lines:
        if "error" in line.lower() or "warning" in line.lower():
            diagnostics.append({
                "message": line.strip(),
                "level": "error" if "error" in line.lower() else "warning",
            })
    
    return diagnostics


def run_single_test(
    skill_meta: Dict[str, Any],
    test_id: str,
    batch_id: int
) -> TestResult:
    """Run a single test for a skill file."""
    skill_name = skill_meta["name"]
    skill_file = get_skill_file(skill_name)
    
    if not skill_file:
        return TestResult(
            test_id=test_id,
            skill_name=skill_name,
            batch_id=batch_id,
            complexity=skill_meta.get("complexity", "medium"),
            status="skipped",
            duration_ms=0,
            targets_tested=[],
            targets_passed=[],
            targets_failed=[],
            diagnostics=[],
            error_message=f"Skill file not found: {skill_name}",
        )
    
    log_info(f"Testing {skill_name}...")
    
    start_time = time.time()
    
    # Run check first
    check_result = check_skill(skill_file)
    
    # Run compile
    compile_result = compile_skill(skill_file)
    
    duration_ms = (time.time() - start_time) * 1000
    
    # Parse results
    diagnostics = parse_diagnostics(check_result.stdout + check_result.stderr)
    
    # Determine status
    if compile_result.returncode == 0:
        status = "passed"
        targets_passed = ["claude", "codex", "gemini"]
        targets_failed = []
        log_success(f"{skill_name} passed ({duration_ms:.0f}ms)")
    else:
        status = "failed"
        targets_passed = []
        targets_failed = ["claude", "codex", "gemini"]
        log_fail(f"{skill_name} failed: {compile_result.stderr[:100]}")
    
    # Find output files
    output_dir = RESULTS_DIR / "compiled"
    output_files = []
    if output_dir.exists():
        for ext in ["claude.md", "codex.json", "gemini.md"]:
            output_file = output_dir / f"{skill_name}.{ext}"
            if output_file.exists():
                output_files.append(str(output_file.relative_to(PROJECT_ROOT)))
    
    return TestResult(
        test_id=test_id,
        skill_name=skill_name,
        batch_id=batch_id,
        complexity=skill_meta.get("complexity", "medium"),
        status=status,
        duration_ms=duration_ms,
        targets_tested=["claude", "codex", "gemini"],
        targets_passed=targets_passed,
        targets_failed=targets_failed,
        diagnostics=diagnostics,
        error_message=compile_result.stderr if status == "failed" else None,
        output_files=output_files,
    )


def run_batch(
    batch_id: int,
    manifest: Dict[str, Any],
    limit: Optional[int] = None,
    test_interval: float = DEFAULT_TEST_INTERVAL
) -> BatchResult:
    """Run all tests for a specific batch."""
    batch_names = {
        1: "Official Anthropic Skills",
        2: "Development Tools",
        3: "Business & Creative",
        4: "SaaS Automation - Communication",
        5: "SaaS Automation - Project Management",
        6: "SaaS Automation - DevOps",
        7: "SaaS Automation - Commerce",
        8: "Productivity Tools",
    }
    
    batch_name = batch_names.get(batch_id, f"Batch {batch_id}")
    
    log_info(f"\n{'='*60}")
    log_info(f"Running Batch {batch_id}: {batch_name}")
    log_info(f"{'='*60}\n")
    
    # Get skills for this batch
    skills = get_skills_for_batch(batch_id, manifest)
    
    if not skills:
        log_warn(f"No skills found for batch {batch_id}")
        return BatchResult(
            batch_id=batch_id,
            batch_name=batch_name,
            total_tests=0,
            passed=0,
            failed=0,
            skipped=0,
            errors=0,
            pass_rate=0.0,
            avg_duration_ms=0.0,
            total_duration_s=0.0,
            results=[],
            started_at=datetime.now().isoformat(),
            completed_at=datetime.now().isoformat(),
        )
    
    # Apply limit
    if limit and len(skills) > limit:
        skills = skills[:limit]
        log_info(f"Limiting to {limit} tests")
    
    # Run tests
    results = []
    passed = 0
    failed = 0
    skipped = 0
    errors = 0
    total_duration = 0.0
    
    batch_start = time.time()
    
    for i, skill_meta in enumerate(skills):
        test_id = f"B{batch_id}-{i+1:03d}"
        
        try:
            result = run_single_test(skill_meta, test_id, batch_id)
            results.append(result)
            
            if result.status == "passed":
                passed += 1
            elif result.status == "failed":
                failed += 1
            elif result.status == "skipped":
                skipped += 1
            else:
                errors += 1
            
            total_duration += result.duration_ms
            
            # Pause between tests
            if i < len(skills) - 1:
                time.sleep(test_interval)
                
        except Exception as e:
            log_error(f"Test {test_id} crashed: {e}")
            errors += 1
            results.append(TestResult(
                test_id=test_id,
                skill_name=skill_meta.get("name", "unknown"),
                batch_id=batch_id,
                complexity=skill_meta.get("complexity", "medium"),
                status="error",
                duration_ms=0,
                targets_tested=[],
                targets_passed=[],
                targets_failed=[],
                diagnostics=[],
                error_message=str(e),
            ))
    
    batch_duration = time.time() - batch_start
    
    # Calculate pass rate
    total_tests = passed + failed + skipped + errors
    pass_rate = (passed / total_tests * 100) if total_tests > 0 else 0.0
    avg_duration = total_duration / len(results) if results else 0.0
    
    # Print batch summary
    log_info(f"\n{'='*40}")
    log_info(f"Batch {batch_id} Summary")
    log_info(f"{'='*40}")
    log_info(f"Total: {total_tests} | Passed: {passed} | Failed: {failed} | Skipped: {skipped}")
    log_info(f"Pass Rate: {pass_rate:.1f}%")
    log_info(f"Avg Duration: {avg_duration:.0f}ms")
    log_info(f"Total Time: {batch_duration:.1f}s")
    
    return BatchResult(
        batch_id=batch_id,
        batch_name=batch_name,
        total_tests=total_tests,
        passed=passed,
        failed=failed,
        skipped=skipped,
        errors=errors,
        pass_rate=pass_rate,
        avg_duration_ms=avg_duration,
        total_duration_s=batch_duration,
        results=results,
        started_at=datetime.now().isoformat(),
        completed_at=datetime.now().isoformat(),
    )


# ============================================================================
# Report Generation
# ============================================================================

def generate_report(batch_results: List[BatchResult]) -> TestReport:
    """Generate a comprehensive test report."""
    total_tests = sum(b.total_tests for b in batch_results)
    total_passed = sum(b.passed for b in batch_results)
    total_failed = sum(b.failed for b in batch_results)
    
    overall_pass_rate = (total_passed / total_tests * 100) if total_tests > 0 else 0.0
    
    # Summary by complexity
    complexity_summary = {}
    for batch in batch_results:
        for result in batch.results:
            comp = result.complexity
            if comp not in complexity_summary:
                complexity_summary[comp] = {"passed": 0, "failed": 0, "total": 0}
            complexity_summary[comp]["total"] += 1
            if result.status == "passed":
                complexity_summary[comp]["passed"] += 1
            else:
                complexity_summary[comp]["failed"] += 1
    
    # Summary by source (from manifest)
    source_summary = {}
    manifest = load_manifest()
    if manifest:
        for skill in manifest.get("skills", []):
            source = skill.get("source", "unknown")
            if source not in source_summary:
                source_summary[source] = {"passed": 0, "failed": 0, "total": 0}
            source_summary[source]["total"] += 1
    
    # Performance metrics
    all_durations = [r.duration_ms for b in batch_results for r in b.results]
    performance_metrics = {
        "avg_duration_ms": sum(all_durations) / len(all_durations) if all_durations else 0,
        "min_duration_ms": min(all_durations) if all_durations else 0,
        "max_duration_ms": max(all_durations) if all_durations else 0,
        "p50_duration_ms": sorted(all_durations)[len(all_durations)//2] if all_durations else 0,
        "p95_duration_ms": sorted(all_durations)[int(len(all_durations)*0.95)] if all_durations else 0,
    }
    
    return TestReport(
        version="1.0",
        generated_at=datetime.now().isoformat(),
        total_batches=len(batch_results),
        total_tests=total_tests,
        total_passed=total_passed,
        total_failed=total_failed,
        overall_pass_rate=overall_pass_rate,
        batch_results=[asdict(b) for b in batch_results],
        summary_by_complexity=complexity_summary,
        summary_by_source=source_summary,
        performance_metrics=performance_metrics,
    )


def save_report(report: TestReport, output_file: Path) -> None:
    """Save report to JSON file."""
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(asdict(report), f, indent=2)
    log_info(f"Report saved to {output_file}")


def generate_markdown_report(report: TestReport) -> str:
    """Generate a markdown version of the report."""
    md = []
    
    md.append("# Nexa Skill Compiler - Batch Test Report")
    md.append("")
    md.append(f"**Generated**: {report.generated_at}")
    md.append(f"**Version**: {report.version}")
    md.append("")
    
    md.append("## Executive Summary")
    md.append("")
    md.append(f"| Metric | Value |")
    md.append(f"|--------|-------|")
    md.append(f"| Total Tests | {report.total_tests} |")
    md.append(f"| Passed | {report.total_passed} |")
    md.append(f"| Failed | {report.total_failed} |")
    md.append(f"| Pass Rate | {report.overall_pass_rate:.1f}% |")
    md.append(f"| Batches Run | {report.total_batches} |")
    md.append("")
    
    md.append("## Performance Metrics")
    md.append("")
    md.append(f"| Metric | Value |")
    md.append(f"|--------|-------|")
    md.append(f"| Average Duration | {report.performance_metrics['avg_duration_ms']:.0f}ms |")
    md.append(f"| Min Duration | {report.performance_metrics['min_duration_ms']:.0f}ms |")
    md.append(f"| Max Duration | {report.performance_metrics['max_duration_ms']:.0f}ms |")
    md.append(f"| P50 Duration | {report.performance_metrics['p50_duration_ms']:.0f}ms |")
    md.append(f"| P95 Duration | {report.performance_metrics['p95_duration_ms']:.0f}ms |")
    md.append("")
    
    md.append("## Batch Results")
    md.append("")
    
    for batch in report.batch_results:
        md.append(f"### Batch {batch['batch_id']}: {batch['batch_name']}")
        md.append("")
        md.append(f"- **Total**: {batch['total_tests']}")
        md.append(f"- **Passed**: {batch['passed']}")
        md.append(f"- **Failed**: {batch['failed']}")
        md.append(f"- **Pass Rate**: {batch['pass_rate']:.1f}%")
        md.append(f"- **Avg Duration**: {batch['avg_duration_ms']:.0f}ms")
        md.append("")
        
        # Individual results
        md.append("| Test ID | Skill | Status | Duration |")
        md.append("|---------|-------|--------|----------|")
        
        for result in batch['results']:
            status_icon = "✓" if result['status'] == 'passed' else "✗"
            md.append(f"| {result['test_id']} | {result['skill_name']} | {status_icon} {result['status']} | {result['duration_ms']:.0f}ms |")
        md.append("")
    
    md.append("## Summary by Complexity")
    md.append("")
    md.append("| Complexity | Passed | Failed | Total | Pass Rate |")
    md.append("|------------|--------|--------|-------|-----------|")
    
    for comp, stats in report.summary_by_complexity.items():
        rate = (stats['passed'] / stats['total'] * 100) if stats['total'] > 0 else 0
        md.append(f"| {comp} | {stats['passed']} | {stats['failed']} | {stats['total']} | {rate:.1f}% |")
    md.append("")
    
    md.append("## Failed Tests Analysis")
    md.append("")
    
    failed_tests = []
    for batch in report.batch_results:
        for result in batch['results']:
            if result['status'] == 'failed':
                failed_tests.append(result)
    
    if failed_tests:
        md.append("| Test ID | Skill | Error Message |")
        md.append("|---------|-------|---------------|")
        
        for test in failed_tests[:20]:  # Limit to 20 for readability
            error = test.get('error_message', 'Unknown error')[:50]
            md.append(f"| {test['test_id']} | {test['skill_name']} | {error}... |")
        md.append("")
    else:
        md.append("No failed tests! 🎉")
        md.append("")
    
    return "\n".join(md)


# ============================================================================
# Main Entry Point
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Run batch tests for Nexa Skill Compiler"
    )
    parser.add_argument(
        "--batch",
        type=int,
        default=None,
        help="Run only a specific batch (1-8)"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all batches progressively"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit number of tests per batch"
    )
    parser.add_argument(
        "--interval",
        type=float,
        default=DEFAULT_TEST_INTERVAL,
        help="Seconds to pause between tests"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output report file path"
    )
    
    args = parser.parse_args()
    
    # Create directories
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    (RESULTS_DIR / "compiled").mkdir(parents=True, exist_ok=True)
    
    # Load manifest
    manifest = load_manifest()
    if not manifest:
        log_error("No manifest found. Run fetch_real_skills.py first.")
        sys.exit(1)
    
    log_info(f"Loaded manifest with {manifest['total_skills']} skills")
    
    # Determine which batches to run
    if args.batch:
        batch_ids = [args.batch]
    elif args.all:
        batch_ids = sorted([int(k) for k in manifest.get("batches", {}).keys()])
    else:
        # Default: run batch 1
        batch_ids = [1]
    
    # Run batches
    batch_results = []
    
    for batch_id in batch_ids:
        result = run_batch(
            batch_id,
            manifest,
            limit=args.limit,
            test_interval=args.interval
        )
        batch_results.append(result)
        
        # Pause between batches
        if batch_id < max(batch_ids):
            log_info("Pausing 10 seconds before next batch...")
            time.sleep(10)
    
    # Generate report
    report = generate_report(batch_results)
    
    # Save JSON report
    report_file = args.output or str(REPORTS_DIR / f"batch_test_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json")
    save_report(report, Path(report_file))
    
    # Save markdown report
    md_report = generate_markdown_report(report)
    md_file = Path(report_file).with_suffix(".md")
    with open(md_file, "w", encoding="utf-8") as f:
        f.write(md_report)
    log_info(f"Markdown report saved to {md_file}")
    
    # Final summary
    log_info(f"\n{'='*60}")
    log_info("TEST RUN COMPLETE")
    log_info(f"{'='*60}")
    log_info(f"Total tests: {report.total_tests}")
    log_info(f"Passed: {report.total_passed}")
    log_info(f"Failed: {report.total_failed}")
    log_info(f"Pass rate: {report.overall_pass_rate:.1f}%")
    
    # Exit with appropriate code
    if report.total_failed > 0:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()