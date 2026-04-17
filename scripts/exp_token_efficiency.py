#!/usr/bin/env python3
"""Experiment 2: Token Efficiency (P1)

Uses cross-platform compiled outputs from Experiment 1 to measure
token counts across platforms and compute savings percentages.
"""

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

# ── Configuration ──────────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CROSS_PLATFORM_DIR = PROJECT_ROOT / "experiments" / "exp_cross_platform"
OUTPUT_BASE = PROJECT_ROOT / "experiments" / "exp_token_efficiency"
TARGETS = ["claude", "codex", "gemini", "kimi"]

# ── Token counting ─────────────────────────────────────────────────────────

try:
    import tiktoken
    ENCODING = tiktoken.get_encoding("cl100k_base")
except ImportError:
    print("tiktoken not found. Installing...")
    subprocess_result = __import__("subprocess").run(
        [sys.executable, "-m", "pip", "install", "tiktoken"],
        capture_output=True, text=True, timeout=60,
    )
    if subprocess_result.returncode != 0:
        print(f"Failed to install tiktoken: {subprocess_result.stderr}")
        sys.exit(1)
    import tiktoken
    ENCODING = tiktoken.get_encoding("cl100k_base")


def count_tokens(text: str) -> int:
    """Count tokens using tiktoken cl100k_base encoding."""
    return len(ENCODING.encode(text))


def classify_complexity(line_count: int) -> str:
    """Classify skill complexity based on source line count."""
    if line_count < 50:
        return "simple"
    elif line_count <= 150:
        return "medium"
    else:
        return "complex"


def load_cross_platform_results() -> dict:
    """Load results from Experiment 1."""
    results_path = CROSS_PLATFORM_DIR / "results.json"
    if not results_path.exists():
        print(f"ERROR: Cross-platform results not found at {results_path}")
        print("Run Experiment 1 (exp_cross_platform.py) first.")
        sys.exit(1)

    with open(results_path, "r", encoding="utf-8") as f:
        return json.load(f)


def find_output_file(target_dir: Path, skill_name: str, target: str) -> Path | None:
    """Find the compiled output file for a skill/target combination."""
    # The output dir structure is: exp_cross_platform/{target}/{skill_name}/
    skill_output_dir = target_dir / target / skill_name

    if not skill_output_dir.exists():
        return None

    # Find content files (exclude manifest.json)
    files = [f for f in skill_output_dir.iterdir() if f.name != "manifest.json" and f.is_file()]

    if not files:
        return None

    # For claude, prefer .xml; for others, prefer .md
    if target == "claude":
        xml_files = [f for f in files if f.suffix == ".xml"]
        if xml_files:
            return xml_files[0]

    if target in ("codex", "gemini", "kimi"):
        md_files = [f for f in files if f.suffix == ".md"]
        if md_files:
            return md_files[0]

    # Fallback: return any content file
    return files[0] if files else None


def generate_report(data: dict) -> str:
    """Generate markdown report for token efficiency."""
    lines = []
    lines.append("# Experiment 2: Token Efficiency Analysis\n")
    lines.append(f"**Date**: {data['timestamp']}")
    lines.append(f"**NSC Version**: `{data['nsc_version']}`")
    lines.append(f"**Skills Analyzed**: {data['config']['num_skills']}")
    lines.append(f"**Encoding**: cl100k_base (GPT-4/ChatGPT)")
    lines.append("")

    # Aggregate summary
    lines.append("## Aggregate Token Statistics\n")
    lines.append("| Platform | Avg Tokens | Avg Savings vs Source | Avg Savings vs Claude |")
    lines.append("|----------|-----------|---------------------|----------------------|")

    agg = data["aggregate"]
    for target in TARGETS:
        t_data = agg[target]
        avg_tokens = t_data["avg_tokens"]
        savings_vs_source = t_data["avg_savings_pct_vs_source"]
        savings_vs_claude = t_data["avg_savings_pct_vs_claude"]
        lines.append(f"| {target} | {avg_tokens:.0f} | {savings_vs_source:.1f}% | {savings_vs_claude:.1f}% |")
    lines.append("")

    # Source token stats
    lines.append(f"**Source (original .md) avg tokens**: {agg['source']['avg_tokens']:.0f}")
    lines.append("")

    # Complexity-grouped analysis
    lines.append("## Token Savings by Complexity\n")
    lines.append("| Complexity | # Skills | Source Avg | Claude Avg | Codex Avg | Codex vs Source | Codex vs Claude |")
    lines.append("|-----------|---------|------------|-----------|----------|----------------|----------------|")

    for comp in ["simple", "medium", "complex"]:
        group = data["complexity_groups"][comp]
        if group["count"] == 0:
            lines.append(f"| {comp} | 0 | - | - | - | - | - |")
        else:
            lines.append(
                f"| {comp} | {group['count']} | {group['avg_source_tokens']:.0f} "
                f"| {group['avg_claude_tokens']:.0f} | {group['avg_codex_tokens']:.0f} "
                f"| {group['avg_savings_pct_vs_source']:.1f}% | {group['avg_savings_pct_vs_claude']:.1f}% |"
            )
    lines.append("")

    # Per-skill detail table
    lines.append("## Per-Skill Token Counts\n")
    lines.append("| Skill | Complexity | Source | claude | codex | gemini | kimi | Codex Savings vs Source | Codex Savings vs Claude |")
    lines.append("|-------|-----------|--------|--------|-------|--------|------|------------------------|------------------------|")

    for r in data["results"]:
        source_t = r["source_tokens"]
        tokens = r["platform_tokens"]
        savings_vs_source = r["savings_pct_vs_source"]
        savings_vs_claude = r["savings_pct_vs_claude"]
        lines.append(
            f"| {r['skill_name']} | {r['complexity']} | {source_t} "
            f"| {tokens.get('claude', 'N/A')} | {tokens.get('codex', 'N/A')} "
            f"| {tokens.get('gemini', 'N/A')} | {tokens.get('kimi', 'N/A')} "
            f"| {savings_vs_source:.1f}% | {savings_vs_claude:.1f}% |"
        )
    lines.append("")

    return "\n".join(lines)


def main():
    print("=" * 60)
    print("Experiment 2: Token Efficiency Analysis")
    print("=" * 60)

    # Load cross-platform results
    cp_data = load_cross_platform_results()
    print(f"Loaded cross-platform results: {cp_data['config']['num_skills']} skills")

    nsc_version = cp_data["nsc_version"]
    OUTPUT_BASE.mkdir(parents=True, exist_ok=True)

    # Process each skill
    results = []
    for cp_result in cp_data["results"]:
        skill_name = cp_result["skill_name"]
        source_lines = cp_result["source_lines"]
        complexity = classify_complexity(source_lines)
        source_path = PROJECT_ROOT / cp_result["source_path"]

        # Count source tokens
        source_tokens = 0
        if source_path.exists():
            source_text = source_path.read_text(encoding="utf-8")
            source_tokens = count_tokens(source_text)
        else:
            print(f"  WARNING: Source file not found: {source_path}")

        # Count tokens for each platform output
        platform_tokens = {}
        for target in TARGETS:
            output_file = find_output_file(CROSS_PLATFORM_DIR, skill_name, target)
            if output_file and output_file.exists():
                output_text = output_file.read_text(encoding="utf-8")
                platform_tokens[target] = count_tokens(output_text)
            else:
                # Check if compilation was successful but we couldn't find file
                comp = cp_result["compilations"][target]
                if comp["success"]:
                    # Try to find the output from the compilation output_dir field
                    alt_dir = Path(comp.get("output_file", ""))
                    if alt_dir.exists():
                        alt_text = alt_dir.read_text(encoding="utf-8")
                        platform_tokens[target] = count_tokens(alt_text)
                    else:
                        platform_tokens[target] = 0
                        print(f"  WARNING: Output file missing for {skill_name}/{target}")
                else:
                    platform_tokens[target] = 0

        # Calculate savings
        codex_tokens = platform_tokens.get("codex", 0)
        claude_tokens = platform_tokens.get("claude", 0)

        savings_pct_vs_source = (
            ((source_tokens - codex_tokens) / source_tokens) * 100
            if source_tokens > 0 and codex_tokens > 0
            else 0.0
        )
        savings_pct_vs_claude = (
            ((claude_tokens - codex_tokens) / claude_tokens) * 100
            if claude_tokens > 0 and codex_tokens > 0
            else 0.0
        )

        result_entry = {
            "skill_name": skill_name,
            "source_lines": source_lines,
            "complexity": complexity,
            "source_tokens": source_tokens,
            "platform_tokens": platform_tokens,
            "savings_pct_vs_source": round(savings_pct_vs_source, 2),
            "savings_pct_vs_claude": round(savings_pct_vs_claude, 2),
        }
        results.append(result_entry)

        print(f"  {skill_name}: source={source_tokens}, claude={platform_tokens.get('claude', 0)}, "
              f"codex={platform_tokens.get('codex', 0)}, savings_vs_source={savings_pct_vs_source:.1f}%")

    # Compute aggregate statistics
    # Filter to skills where at least one platform succeeded
    valid_results = [r for r in results if any(r["platform_tokens"].get(t, 0) > 0 for t in TARGETS)]

    source_avg = sum(r["source_tokens"] for r in valid_results) / len(valid_results) if valid_results else 0

    aggregate = {"source": {"avg_tokens": round(source_avg, 2)}}
    for target in TARGETS:
        target_tokens_list = [r["platform_tokens"].get(target, 0) for r in valid_results if r["platform_tokens"].get(target, 0) > 0]
        avg_tokens = sum(target_tokens_list) / len(target_tokens_list) if target_tokens_list else 0

        # Savings vs source
        savings_vs_source_list = []
        for r in valid_results:
            s = r["source_tokens"]
            t = r["platform_tokens"].get(target, 0)
            if s > 0 and t > 0:
                savings_vs_source_list.append(((s - t) / s) * 100)

        avg_savings_vs_source = sum(savings_vs_source_list) / len(savings_vs_source_list) if savings_vs_source_list else 0

        # Savings vs claude (only meaningful for non-claude targets)
        savings_vs_claude_list = []
        if target != "claude":
            for r in valid_results:
                claude_t = r["platform_tokens"].get("claude", 0)
                this_t = r["platform_tokens"].get(target, 0)
                if claude_t > 0 and this_t > 0:
                    savings_vs_claude_list.append(((claude_t - this_t) / claude_t) * 100)

        avg_savings_vs_claude = sum(savings_vs_claude_list) / len(savings_vs_claude_list) if savings_vs_claude_list else 0

        aggregate[target] = {
            "avg_tokens": round(avg_tokens, 2),
            "avg_savings_pct_vs_source": round(avg_savings_vs_source, 2),
            "avg_savings_pct_vs_claude": round(avg_savings_vs_claude, 2),
        }

    # Compute complexity-grouped statistics
    complexity_groups = {}
    for comp in ["simple", "medium", "complex"]:
        group_results = [r for r in valid_results if r["complexity"] == comp]
        count = len(group_results)

        if count == 0:
            complexity_groups[comp] = {
                "count": 0,
                "avg_source_tokens": 0,
                "avg_claude_tokens": 0,
                "avg_codex_tokens": 0,
                "avg_savings_pct_vs_source": 0,
                "avg_savings_pct_vs_claude": 0,
            }
            continue

        avg_source = sum(r["source_tokens"] for r in group_results) / count
        avg_claude = sum(r["platform_tokens"].get("claude", 0) for r in group_results) / count
        avg_codex = sum(r["platform_tokens"].get("codex", 0) for r in group_results) / count

        savings_vs_source_list = []
        savings_vs_claude_list = []
        for r in group_results:
            if r["source_tokens"] > 0 and r["platform_tokens"].get("codex", 0) > 0:
                savings_vs_source_list.append(r["savings_pct_vs_source"])
            if r["platform_tokens"].get("claude", 0) > 0 and r["platform_tokens"].get("codex", 0) > 0:
                savings_vs_claude_list.append(r["savings_pct_vs_claude"])

        complexity_groups[comp] = {
            "count": count,
            "avg_source_tokens": round(avg_source, 2),
            "avg_claude_tokens": round(avg_claude, 2),
            "avg_codex_tokens": round(avg_codex, 2),
            "avg_savings_pct_vs_source": round(
                sum(savings_vs_source_list) / len(savings_vs_source_list), 2
            ) if savings_vs_source_list else 0,
            "avg_savings_pct_vs_claude": round(
                sum(savings_vs_claude_list) / len(savings_vs_claude_list), 2
            ) if savings_vs_claude_list else 0,
        }

    # Build full data structure
    data = {
        "experiment": "exp_token_efficiency",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "nsc_version": nsc_version,
        "config": {
            "num_skills": len(results),
            "targets": TARGETS,
            "encoding": "cl100k_base",
        },
        "results": results,
        "aggregate": aggregate,
        "complexity_groups": complexity_groups,
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
    print(f"  Source avg tokens: {source_avg:.0f}")
    for target in TARGETS:
        t_data = aggregate[target]
        print(f"  {target}: avg={t_data['avg_tokens']:.0f} tokens, "
              f"savings vs source={t_data['avg_savings_pct_vs_source']:.1f}%, "
              f"savings vs claude={t_data['avg_savings_pct_vs_claude']:.1f}%")

    # Print complexity breakdown
    print("\n  Complexity breakdown:")
    for comp in ["simple", "medium", "complex"]:
        g = complexity_groups[comp]
        if g["count"] > 0:
            print(f"    {comp} ({g['count']} skills): source={g['avg_source_tokens']:.0f}, "
                  f"claude={g['avg_claude_tokens']:.0f}, codex={g['avg_codex_tokens']:.0f}, "
                  f"codex vs source={g['avg_savings_pct_vs_source']:.1f}%")


if __name__ == "__main__":
    main()