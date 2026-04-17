#!/usr/bin/env python3
"""Experiment 1: Cross-Platform Compilation (P0)

Compiles all real_corpus skills to 4 target platforms (claude, codex, gemini, kimi),
measures success rates, output validity, and compilation timing.
"""

import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

# ── Configuration ──────────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).resolve().parent.parent
NSC_BIN = PROJECT_ROOT / "target" / "release" / "nexa-skill"
REAL_CORPUS_DIR = PROJECT_ROOT / "tests" / "fixtures" / "real_corpus"
OUTPUT_BASE = PROJECT_ROOT / "experiments" / "exp_cross_platform"
TARGETS = ["claude", "codex", "gemini", "kimi"]
TIMEOUT_SEC = 30

# Key sections expected in compiled output per target
EXPECTED_SECTIONS = {
    "claude": ["<agent_skill>", "<metadata>", "<intent>", "<execution_steps>"],
    "codex": ["# ", "## Intent", "## Procedures", "## Constraints"],
    "gemini": ["# ", "## Intent", "## Procedures", "## Constraints"],
    "kimi": ["# ", "## Intent", "## Procedures", "## Constraints"],
}


def get_nsc_version() -> str:
    """Get NSC version from git hash."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, timeout=5,
            cwd=str(PROJECT_ROOT),
        )
        return result.stdout.strip() if result.returncode == 0 else "unknown"
    except Exception:
        return "unknown"


def scan_skill_files() -> list[Path]:
    """Scan all .md files in real_corpus, excluding .meta.json companion files."""
    md_files = sorted(REAL_CORPUS_DIR.glob("*.md"))
    # Exclude any files that are just metadata companions
    skills = [f for f in md_files if not f.name.endswith(".meta.json")]
    return skills


def classify_complexity(line_count: int) -> str:
    """Classify skill complexity based on source line count."""
    if line_count < 50:
        return "simple"
    elif line_count <= 150:
        return "medium"
    else:
        return "complex"


def verify_output(output_dir: Path, skill_name: str, target: str) -> dict:
    """Verify compiled output file exists, is non-empty, and contains key sections."""
    info = {
        "output_valid": False,
        "output_size_bytes": 0,
        "output_file": "",
        "sections_found": [],
        "sections_missing": [],
    }

    # Determine expected output filename per target
    if target == "claude":
        expected_file = output_dir / f"{skill_name}.xml"
    elif target == "codex":
        expected_file = output_dir / f"{skill_name}.md"
    elif target == "gemini":
        expected_file = output_dir / f"{skill_name}.md"
    elif target == "kimi":
        expected_file = output_dir / f"{skill_name}.md"
    else:
        return info

    # Also check if any file exists in the directory (in case naming differs)
    if not expected_file.exists():
        # Fallback: find any non-manifest file in the output dir
        files_in_dir = list(output_dir.iterdir()) if output_dir.exists() else []
        content_files = [f for f in files_in_dir if f.name != "manifest.json"]
        if content_files:
            expected_file = content_files[0]

    if not expected_file.exists():
        info["sections_missing"] = EXPECTED_SECTIONS.get(target, [])
        return info

    info["output_file"] = str(expected_file.relative_to(PROJECT_ROOT))
    info["output_size_bytes"] = expected_file.stat().st_size

    if info["output_size_bytes"] == 0:
        info["sections_missing"] = EXPECTED_SECTIONS.get(target, [])
        return info

    # Read content and check for key sections
    try:
        content = expected_file.read_text(encoding="utf-8")
    except Exception:
        info["sections_missing"] = EXPECTED_SECTIONS.get(target, [])
        return info

    expected = EXPECTED_SECTIONS.get(target, [])
    found = []
    missing = []
    for section in expected:
        if section in content:
            found.append(section)
        else:
            missing.append(section)

    info["sections_found"] = found
    info["sections_missing"] = missing
    info["output_valid"] = len(found) >= 2  # At least 2 key sections present

    return info


def compile_skill(skill_path: Path, target: str, output_dir: Path) -> dict:
    """Compile a single skill to a target platform and record results."""
    skill_name = skill_path.stem
    target_output_dir = output_dir / target / skill_name

    result = {
        "success": False,
        "output_size_bytes": 0,
        "elapsed_ms": 0.0,
        "output_valid": False,
        "output_file": "",
        "sections_found": [],
        "sections_missing": [],
        "error_message": "",
    }

    cmd = [
        str(NSC_BIN),
        "build",
        str(skill_path),
        f"--{target}",
        "-o", str(target_output_dir),
        "--force",
    ]

    start_time = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True, text=True, timeout=TIMEOUT_SEC,
            cwd=str(PROJECT_ROOT),
        )
        elapsed_ms = (time.perf_counter() - start_time) * 1000.0
        result["elapsed_ms"] = round(elapsed_ms, 2)

        if proc.returncode == 0:
            result["success"] = True
            # Verify output
            verification = verify_output(target_output_dir, skill_name, target)
            result["output_size_bytes"] = verification["output_size_bytes"]
            result["output_valid"] = verification["output_valid"]
            result["output_file"] = verification["output_file"]
            result["sections_found"] = verification["sections_found"]
            result["sections_missing"] = verification["sections_missing"]
        else:
            result["error_message"] = proc.stderr.strip() if proc.stderr else proc.stdout.strip()

    except subprocess.TimeoutExpired:
        elapsed_ms = (time.perf_counter() - start_time) * 1000.0
        result["elapsed_ms"] = round(elapsed_ms, 2)
        result["error_message"] = f"Timeout after {TIMEOUT_SEC}s"
    except Exception as e:
        elapsed_ms = (time.perf_counter() - start_time) * 1000.0
        result["elapsed_ms"] = round(elapsed_ms, 2)
        result["error_message"] = str(e)

    return result


def generate_report(data: dict) -> str:
    """Generate markdown report matching paper format."""
    lines = []
    lines.append("# Experiment 1: Cross-Platform Compilation\n")
    lines.append(f"**Date**: {data['timestamp']}")
    lines.append(f"**NSC Version**: `{data['nsc_version']}`")
    lines.append(f"**Skills Tested**: {data['config']['num_skills']}")
    lines.append(f"**Targets**: {', '.join(data['config']['targets'])}")
    lines.append("")

    # Summary table
    lines.append("## Aggregate Results\n")
    lines.append("| Platform | Success Rate | Avg Time (ms) | Avg Output Size (bytes) | Valid Output Rate |")
    lines.append("|----------|-------------|---------------|------------------------|-------------------|")

    for target in TARGETS:
        summary = data["summary"][target]
        lines.append(
            f"| {target} | {summary['success_rate']} | {summary['avg_elapsed_ms']:.1f} "
            f"| {summary['avg_output_size_bytes']:.0f} | {summary['valid_output_rate']} |"
        )
    lines.append("")

    # Per-skill detail table
    lines.append("## Per-Skill Results\n")
    lines.append("| Skill | Complexity | Lines | claude | codex | gemini | kimi |")
    lines.append("|-------|-----------|-------|--------|-------|--------|------|")

    for r in data["results"]:
        compilations = r["compilations"]
        claude_status = "✅" if compilations["claude"]["success"] else "❌"
        codex_status = "✅" if compilations["codex"]["success"] else "❌"
        gemini_status = "✅" if compilations["gemini"]["success"] else "❌"
        kimi_status = "✅" if compilations["kimi"]["success"] else "❌"
        lines.append(
            f"| {r['skill_name']} | {r['complexity']} | {r['source_lines']} "
            f"| {claude_status} | {codex_status} | {gemini_status} | {kimi_status} |"
        )
    lines.append("")

    # Timing detail
    lines.append("## Compilation Timing (ms)\n")
    lines.append("| Skill | claude | codex | gemini | kimi |")
    lines.append("|-------|--------|-------|--------|------|")
    for r in data["results"]:
        compilations = r["compilations"]
        lines.append(
            f"| {r['skill_name']} "
            f"| {compilations['claude']['elapsed_ms']:.1f} "
            f"| {compilations['codex']['elapsed_ms']:.1f} "
            f"| {compilations['gemini']['elapsed_ms']:.1f} "
            f"| {compilations['kimi']['elapsed_ms']:.1f} |"
        )
    lines.append("")

    # Output size detail
    lines.append("## Output Size (bytes)\n")
    lines.append("| Skill | claude | codex | gemini | kimi |")
    lines.append("|-------|--------|-------|--------|------|")
    for r in data["results"]:
        compilations = r["compilations"]
        lines.append(
            f"| {r['skill_name']} "
            f"| {compilations['claude']['output_size_bytes']} "
            f"| {compilations['codex']['output_size_bytes']} "
            f"| {compilations['gemini']['output_size_bytes']} "
            f"| {compilations['kimi']['output_size_bytes']} |"
        )
    lines.append("")

    return "\n".join(lines)


def main():
    print("=" * 60)
    print("Experiment 1: Cross-Platform Compilation")
    print("=" * 60)

    # Ensure NSC binary exists
    if not NSC_BIN.exists():
        print(f"ERROR: NSC binary not found at {NSC_BIN}")
        print("Run 'cargo build --release' first.")
        sys.exit(1)

    # Ensure output directory exists
    OUTPUT_BASE.mkdir(parents=True, exist_ok=True)

    # Scan skill files
    skill_files = scan_skill_files()
    print(f"Found {len(skill_files)} skill files in real_corpus")

    # Get version
    nsc_version = get_nsc_version()
    print(f"NSC version: {nsc_version}")

    # Compile each skill to each target
    results = []
    for i, skill_path in enumerate(skill_files):
        skill_name = skill_path.stem
        source_lines = len(skill_path.read_text(encoding="utf-8").splitlines())
        complexity = classify_complexity(source_lines)

        print(f"\n[{i+1}/{len(skill_files)}] Compiling: {skill_name} ({source_lines} lines, {complexity})")

        compilations = {}
        for target in TARGETS:
            print(f"  → {target}...", end=" ", flush=True)
            comp_result = compile_skill(skill_path, target, OUTPUT_BASE)
            compilations[target] = comp_result
            status = "✅" if comp_result["success"] else "❌"
            print(f"{status} ({comp_result['elapsed_ms']:.1f}ms, {comp_result['output_size_bytes']}B)")

        results.append({
            "skill_name": skill_name,
            "source_lines": source_lines,
            "source_path": str(skill_path.relative_to(PROJECT_ROOT)),
            "complexity": complexity,
            "compilations": compilations,
        })

    # Compute summary statistics
    summary = {}
    for target in TARGETS:
        target_results = [r["compilations"][target] for r in results]
        successes = [r for r in target_results if r["success"]]
        valids = [r for r in target_results if r["output_valid"]]

        success_count = len(successes)
        total = len(target_results)

        avg_elapsed = sum(r["elapsed_ms"] for r in target_results) / total if total > 0 else 0
        avg_size = sum(r["output_size_bytes"] for r in successes) / len(successes) if successes else 0

        summary[target] = {
            "success_rate": f"{success_count}/{total}",
            "success_count": success_count,
            "total": total,
            "avg_elapsed_ms": round(avg_elapsed, 2),
            "avg_output_size_bytes": round(avg_size, 2),
            "valid_output_rate": f"{len(valids)}/{total}",
            "valid_count": len(valids),
        }

    # Build full data structure
    data = {
        "experiment": "exp_cross_platform",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "nsc_version": nsc_version,
        "config": {
            "num_skills": len(skill_files),
            "targets": TARGETS,
        },
        "results": results,
        "summary": summary,
    }

    # Write results.json
    results_json_path = OUTPUT_BASE / "results.json"
    with open(results_json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    print(f"\n✅ Results written to {results_json_path}")

    # Write report.md
    report_md_path = OUTPUT_BASE / "report.md"
    report_content = generate_report(data)
    with open(report_md_path, "w", encoding="utf-8") as f:
        f.write(report_content)
    print(f"✅ Report written to {report_md_path}")

    # Print summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    for target in TARGETS:
        s = summary[target]
        print(f"  {target}: {s['success_rate']} success, {s['avg_elapsed_ms']:.1f}ms avg, {s['valid_output_rate']} valid")


if __name__ == "__main__":
    main()