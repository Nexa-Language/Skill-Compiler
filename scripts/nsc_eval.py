#!/usr/bin/env python3
"""
NSC Instruction Adherence Evaluation

Measures whether compiled platform-native formats improve LLM instruction
adherence compared to original SKILL.md format.

Key insight: This is a FORMAT SENSITIVITY experiment, not a speed test.
NSC's value proposition is that platform-native formats (Claude XML, Codex MD,
Gemini MD+YAML, Kimi full MD) lead to better instruction following.

Experimental protocol:
  1. Compile real skills to platform-native formats using NSC CLI
  2. For each skill, run Claude Code twice:
     - Condition A (original): prompt with raw SKILL.md content
     - Condition B (compiled): prompt with compiled Claude XML content
  3. Score outputs on 5 structured rubric dimensions
  4. Compare scores statistically

Usage:
    python scripts/nsc_eval.py --skills-dir tests/fixtures/real_corpus --limit 5
    python scripts/nsc_eval.py --skills-dir tests/fixtures/real_corpus --all
"""

import argparse
import httpx
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict, field
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional, Any

PROJECT_ROOT = Path(__file__).resolve().parent.parent

# Platform → CLI flag and output file extension mapping
# Ref: nexa-skill-core/src/backend/mod.rs TargetPlatform::extension()
PLATFORM_CONFIG = {
    "claude": {"flag": "--claude", "ext": ".xml"},
    "codex":  {"flag": "--codex",  "ext": ".md"},
    "gemini": {"flag": "--gemini", "ext": ".md"},
    "kimi":   {"flag": "--kimi",   "ext": ".md"},
}


# ============================================================================
# Data Structures
# ============================================================================

@dataclass
class RubricScore:
    """Score for a single output on the instruction adherence rubric."""
    procedure_adherence: float  # 0-1: did it follow all procedure steps?
    constraint_respect: float   # 0-1: did it respect security constraints?
    format_compliance: float    # 0-1: was output in expected format?
    completeness: float         # 0-1: did it cover all required sections?
    security_awareness: float   # 0-1: did it acknowledge HITL/security requirements?
    overall: float              # weighted average


@dataclass
class EvalResult:
    """Result of a single evaluation run."""
    skill_name: str
    condition: str  # "original" or "compiled"
    platform: str   # "claude", "codex", etc.
    prompt: str
    raw_output: str
    rubric: RubricScore
    duration_seconds: float
    timestamp: str
    error: Optional[str] = None


@dataclass
class ComparisonResult:
    """Comparison between original and compiled results for one skill."""
    skill_name: str
    original_score: RubricScore
    compiled_score: RubricScore
    score_diff: RubricScore  # compiled - original
    significant_improvement: bool  # overall diff > 0.1
    # Information preservation metrics (C3b)
    original_char_count: int = 0
    compiled_char_count: int = 0
    information_ratio: float = 0.0  # compiled_chars / original_chars


# ============================================================================
# Skill Loading
# ============================================================================

def load_skills(skills_dir: Path, limit: Optional[int] = None) -> List[Dict[str, Any]]:
    """Load skill files from directory, filtering out error/test files."""
    skills = []
    for md_file in sorted(skills_dir.glob("*.md")):
        # Skip error test files and hidden files
        if md_file.name.startswith("error-") or md_file.name.startswith("-"):
            continue

        content = md_file.read_text(encoding="utf-8")
        name = md_file.stem
        skills.append({
            "name": name,
            "path": str(md_file),
            "content": content,
        })

    if limit:
        skills = skills[:limit]
    return skills


# ============================================================================
# NSC Compilation
# ============================================================================

def compile_skill(
    skill_path: str,
    platform: str = "claude",
    output_dir: str = "experiments/compiled",
) -> Optional[str]:
    """Compile a skill using NSC CLI and return the compiled content.

    Uses the actual nexa-skill CLI interface:
      cargo run --bin nexa-skill -- build <input> --<platform> --out-dir <dir> --force
    """
    if platform not in PLATFORM_CONFIG:
        print(f"  ⚠ Unknown platform: {platform}")
        return None

    skill_name = Path(skill_path).stem
    skill_output_dir = f"{output_dir}/{skill_name}"
    platform_flag = PLATFORM_CONFIG[platform]["flag"]
    ext = PLATFORM_CONFIG[platform]["ext"]

    cmd = [
        "cargo", "run", "--bin", "nexa-skill", "--",
        "build", skill_path,
        platform_flag,
        "--out-dir", skill_output_dir,
        "--force",  # overwrite existing outputs
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=60,
            cwd=str(PROJECT_ROOT),
        )

        if result.returncode != 0:
            print(f"  ⚠ Compilation failed for {skill_name}: {result.stderr[:300]}")
            return None

        # Read the compiled output file: {skill_name}{ext}
        compiled_file = Path(skill_output_dir) / f"{skill_name}{ext}"

        if not compiled_file.exists():
            # Fallback: try scanning the directory for any file with the expected extension
            candidates = list(Path(skill_output_dir).glob(f"*{ext}"))
            if candidates:
                compiled_file = candidates[0]
            else:
                print(f"  ⚠ Compiled file not found: {compiled_file}")
                return None

        return compiled_file.read_text(encoding="utf-8")

    except subprocess.TimeoutExpired:
        print(f"  ⚠ Compilation timeout for {skill_name}")
        return None
    except Exception as e:
        print(f"  ⚠ Compilation error for {skill_name}: {e}")
        return None


# ============================================================================
# Claude Code Execution
# ============================================================================

def run_llm_query(
    prompt: str,
    model: str = "glm-5.1",
    mode: str = "api",
    api_type: str = "openai",
    timeout: int = 120,
    max_tokens: int = 2048,
) -> Dict[str, Any]:
    """Query an LLM and return the output.

    Three execution configurations:
      - api + openai: Use OpenAI-compatible API (supports proxy like aihub)
      - api + anthropic: Use Anthropic API directly (needs Anthropic API key)
      - cli: Use Claude Code CLI in headless mode

    Credentials are read from environment variables or .env file.
    """
    if mode == "api":
        if api_type == "openai":
            return _run_via_openai_api(prompt, model, timeout, max_tokens)
        else:
            return _run_via_anthropic_api(prompt, model, timeout, max_tokens)
    else:
        return _run_via_cli(prompt, model, timeout)


def _load_env_vars() -> Dict[str, str]:
    """Load API credentials from .env file if not already in environment."""
    env_vars: Dict[str, str] = {}
    env_file = PROJECT_ROOT / ".env"
    if env_file.exists():
        for line in env_file.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            key = key.strip()
            value = value.strip()
            # Load both Anthropic and OpenAI credentials
            if key in (
                "ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL",
                "OPENAI_API_KEY", "OPENAI_API_BASE", "OPENAI_MODEL_NAME",
            ):
                env_vars[key] = value
    return env_vars


def _run_via_openai_api(
    prompt: str,
    model: str,
    timeout: int,
    max_tokens: int,
) -> Dict[str, Any]:
    """Call OpenAI-compatible API using Python SDK.

    Works with any OpenAI-compatible proxy (e.g., aihub.arcsysu.cn).
    Reads OPENAI_API_KEY and OPENAI_API_BASE from env or .env file.
    """
    from openai import OpenAI

    env_overrides = _load_env_vars()
    api_key = os.environ.get("OPENAI_API_KEY") or env_overrides.get("OPENAI_API_KEY")
    base_url = os.environ.get("OPENAI_API_BASE") or env_overrides.get("OPENAI_API_BASE")

    if not api_key:
        return {
            "success": False,
            "error": "OPENAI_API_KEY not found in environment or .env file",
            "output": "",
        }

    try:
        client = OpenAI(api_key=api_key, base_url=base_url)
        response = client.chat.completions.create(
            model=model,
            max_tokens=max_tokens,
            messages=[{"role": "user", "content": prompt}],
            timeout=timeout,
        )
        output_text = response.choices[0].message.content or ""

        return {
            "success": True,
            "output": output_text,
            "error": None,
            "model": response.model,
            "usage": {
                "input_tokens": response.usage.prompt_tokens if response.usage else 0,
                "output_tokens": response.usage.completion_tokens if response.usage else 0,
            },
        }

    except Exception as e:
        return {"success": False, "error": f"OpenAI API call failed: {e}", "output": ""}


def _run_via_anthropic_api(
    prompt: str,
    model: str,
    timeout: int,
    max_tokens: int,
) -> Dict[str, Any]:
    """Call Anthropic API directly using Python SDK."""
    import anthropic

    env_overrides = _load_env_vars()
    api_key = os.environ.get("ANTHROPIC_API_KEY") or env_overrides.get("ANTHROPIC_API_KEY")
    base_url = os.environ.get("ANTHROPIC_BASE_URL") or env_overrides.get("ANTHROPIC_BASE_URL")

    if not api_key:
        return {
            "success": False,
            "error": "ANTHROPIC_API_KEY not found in environment or .env file",
            "output": "",
        }

    try:
        client = anthropic.Anthropic(
            api_key=api_key,
            base_url=base_url,
            timeout=httpx.Timeout(timeout, connect=10.0),
        )
        message = client.messages.create(
            model=model,
            max_tokens=max_tokens,
            messages=[{"role": "user", "content": prompt}],
        )
        # Extract text content from response
        output_text = ""
        for block in message.content:
            if hasattr(block, "text"):
                output_text += block.text

        return {
            "success": True,
            "output": output_text,
            "error": None,
            "model": message.model,
            "usage": {
                "input_tokens": message.usage.input_tokens,
                "output_tokens": message.usage.output_tokens,
            },
        }

    except anthropic.APIError as e:
        return {"success": False, "error": f"Anthropic API error: {e}", "output": ""}
    except Exception as e:
        return {"success": False, "error": f"Anthropic API call failed: {e}", "output": ""}


def _run_via_cli(prompt: str, model: str, timeout: int) -> Dict[str, Any]:
    """Run Claude Code CLI in headless mode (fallback)."""
    cmd = [
        "claude",
        "-p", prompt,
        "--output-format", "text",
        "--allowedTools", "Read,Write,Edit,Bash",
        "--model", model,
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(PROJECT_ROOT),
        )

        return {
            "success": result.returncode == 0,
            "output": result.stdout,
            "error": result.stderr if result.stderr else None,
            "returncode": result.returncode,
        }

    except subprocess.TimeoutExpired:
        return {"success": False, "error": f"Timeout after {timeout}s", "output": ""}
    except FileNotFoundError:
        return {
            "success": False,
            "error": "'claude' CLI not found. Install Claude Code CLI first.",
            "output": "",
        }
    except Exception as e:
        return {"success": False, "error": str(e), "output": ""}


# ============================================================================
# Rubric Scoring
# ============================================================================

def _is_xml_format(content: str) -> bool:
    """Check if content is in XML format (compiled output)."""
    return content.strip().startswith("<agent_skill>")


def _parse_frontmatter(skill_content: str) -> Dict[str, str]:
    """Extract metadata from skill content.

    Handles both YAML frontmatter (original SKILL.md) and
    XML metadata (compiled Claude XML output).
    """
    if _is_xml_format(skill_content):
        # XML format: extract from <metadata>, <security_level>, <intent> tags
        import re
        data = {}
        name_match = re.search(r"<name>(.*?)</name>", skill_content)
        if name_match:
            data["name"] = name_match.group(1).strip()
        sec_match = re.search(r"<security_level>(.*?)</security_level>", skill_content)
        if sec_match:
            data["security_level"] = sec_match.group(1).strip()
        intent_match = re.search(r"<intent>(.*?)</intent>", skill_content)
        if intent_match:
            data["description"] = intent_match.group(1).strip()
        if "<system_constraint>" in skill_content:
            data["hitl_required"] = "true"
        return data

    # Markdown format: extract YAML frontmatter
    lines = skill_content.split("\n")
    in_frontmatter = False
    data = {}

    for line in lines:
        if line.strip() == "---":
            in_frontmatter = not in_frontmatter
            continue
        if in_frontmatter and ":" in line:
            key, value = line.split(":", 1)
            data[key.strip()] = value.strip().strip('"').strip("'")

    return data


def _extract_procedure_steps(skill_content: str) -> List[str]:
    """Extract procedure steps from skill content.

    Handles both Markdown numbered lists/heading steps and
    XML <step> tags in compiled output.
    """
    if _is_xml_format(skill_content):
        import re
        steps = []
        for match in re.finditer(r"<step[^>]*>(.*?)</step>", skill_content, re.DOTALL):
            step_text = match.group(1).strip()
            # Remove inner XML tags for clean text extraction
            step_clean = re.sub(r"<[^>]+>", "", step_text).strip()
            if step_clean:
                steps.append(step_clean)

        # Extract approach names from <execution_approaches> as additional steps
        for match in re.finditer(r'<approach[^>]*name="([^"]+)"[^>]*>', skill_content):
            approach_name = match.group(1).strip()
            if approach_name:
                steps.append(approach_name)

        # Fallback: when no <execution_steps> tag exists, extract procedure
        # steps from <additional_context> <section> content.  Skills without
        # ## Procedures heading have all procedural content in additional_context.
        # In compiled XML, numbered steps like "1. **Extracts Ads**:" become
        # "Verb Noun: Description" patterns (colon-separated, no digit prefix),
        # often concatenated on one line without separators.
        if not steps and "<execution_steps>" not in skill_content:
            ac_match = re.search(
                r"<additional_context[^>]*>(.*?)</additional_context>",
                skill_content, re.DOTALL
            )
            if ac_match:
                ac_inner = ac_match.group(1)

                # Strategy 1: Extract "Verb Noun:" action patterns from
                # section body text — these are compiled numbered steps.
                # In XML compilation, multi-line step lists are concatenated
                # on one line (e.g., "approachesProvides Insights"), so we
                # must split at lowercase→uppercase word boundaries first.
                for sec in re.finditer(
                    r"<section[^>]*>(.*?)</section>",
                    ac_inner, re.DOTALL,
                ):
                    body_text = re.sub(r"<[^>]+>", "", sec.group(1)).strip()
                    # Split concatenated text at [a-z][A-Z] boundaries
                    # (e.g., "approachesProvides" → "approaches\nProvides",
                    # "hierarchyCRITICAL" → "hierarchy\nCRITICAL").
                    # Hyphens (User-Friendly) are preserved because the
                    # boundary only matches direct lowercase-uppercase adjacency
                    # with no intervening hyphen.
                    expanded = re.sub(
                        r"([a-z])([A-Z])", r"\1\n\2", body_text
                    )
                    # Find all "Title:" action-label patterns (1-5 words
                    # starting with a capital letter before a colon).
                    # Use [ \t] instead of \s to prevent cross-line matching.
                    actions = re.findall(
                        r"([A-Z][a-z]+(?:[ \t][A-Za-z→\-]+){0,4})[ \t]*:",
                        expanded,
                    )
                    for action in actions:
                        action_clean = action.strip()
                        if action_clean and len(action_clean) > 3:
                            steps.append(action_clean)

                # Strategy 2: If no action patterns found, use section titles
                # as procedural anchors (covers skills where procedure content
                # is in section headings like "Basic Enhancement", "Ad Campaign
                # Planning").  Skip generic/meta titles that aren't procedural.
                if not steps:
                    skip_titles = {
                        "overview", "when to use this skill",
                        "what this skill does", "tips",
                        "related use cases", "quick reference",
                        "common use cases", "output formats",
                    }
                    for sec in re.finditer(
                        r"<section[^>]*>(.*?)</section>",
                        ac_inner, re.DOTALL,
                    ):
                        title_match = re.search(
                            r'title="([^"]+)"', sec.group(0)
                        )
                        if title_match:
                            title = title_match.group(1).strip()
                            if title.lower() not in skip_titles and len(title) > 3:
                                steps.append(title)

        return steps

    # Markdown format: extract numbered steps and Step N: patterns
    steps = []
    for line in skill_content.split("\n"):
        stripped = line.strip()
        if stripped and (
            stripped[0].isdigit() and "." in stripped[:4]
            or stripped.lower().startswith("step")
        ):
            steps.append(stripped)
    return steps


def _extract_key_sections(skill_content: str) -> List[str]:
    """Extract section headings/tags that represent required content areas.

    Used for format_compliance scoring — checks whether the reference content
    has structured sections that the output should reflect.
    """
    if _is_xml_format(skill_content):
        sections = []
        tag_to_section = {
            "intent": "description",
            "execution_steps": "procedures",
            "strict_constraints": "constraints",
            "anti_pattern": "constraints",
            "system_constraint": "constraints",
            "examples": "examples",
            "fallbacks": "fallbacks",
            "pre_conditions": "context",
            "context_gathering": "context",
            "permissions": "instructions",
            "mcp_servers": "instructions",
            "additional_context": "description",
            "execution_approaches": "procedures",
        }
        for tag, section_name in tag_to_section.items():
            if f"<{tag}" in skill_content:
                sections.append(section_name)
        return sections

    # Markdown format: extract ## heading sections
    sections = []
    for heading in [
        "Description", "Procedures", "Examples", "Constraints",
        "Fallbacks", "Context", "Triggers", "Instructions",
        "Setup", "Prerequisites", "Pitfalls", "Reference",
    ]:
        if f"## {heading}" in skill_content or f"# {heading}" in skill_content:
            sections.append(heading.lower())
    return sections


def _extract_completeness_keywords(skill_content: str) -> List[str]:
    """Extract substantive topic keywords that an LLM output should cover.

    Unlike _extract_key_sections which returns abstract section names,
    this extracts content-relevant words that the LLM would naturally
    reference when following the skill instructions.
    """
    import re

    if _is_xml_format(skill_content):
        # XML: extract meaningful text from each major section tag
        keywords = []
        # Top-level tags (additional_context handled separately below)
        section_tags = [
            "intent", "execution_steps", "strict_constraints",
            "examples", "fallbacks", "pre_conditions",
            "context_gathering", "permissions",
            "execution_approaches",
        ]
        filler = {"the", "this", "that", "with", "from", "for", "and",
                  "but", "not", "all", "can", "will", "may", "should",
                  "must", "also", "then", "than", "been", "being", "have",
                  "having", "does", "did", "would", "could", "about"}

        for tag in section_tags:
            match = re.search(rf"<{tag}[^>]*>(.*?)</{tag}>", skill_content, re.DOTALL)
            if match:
                text = re.sub(r"<[^>]+>", "", match.group(1)).strip()
                words = [w.lower() for w in text.split()
                         if len(w) > 3 and w.lower() not in filler]
                keywords.extend(words[:3])

        # Recursively extract keywords from <section> sub-tags inside
        # <additional_context>.  Each section contributes 2-3 keywords
        # from its title attribute and body text — this covers skills
        # without ## Procedures whose content is entirely in additional_context.
        ac_match = re.search(
            r"<additional_context[^>]*>(.*?)</additional_context>",
            skill_content, re.DOTALL,
        )
        if ac_match:
            for sec in re.finditer(
                r"<section[^>]*>(.*?)</section>",
                ac_match.group(1), re.DOTALL,
            ):
                # Title attribute keywords (up to 2)
                title_match = re.search(r'title="([^"]+)"', sec.group(0))
                if title_match:
                    title_words = [w.lower() for w in title_match.group(1).split()
                                   if len(w) > 3 and w.lower() not in filler]
                    keywords.extend(title_words[:2])
                # Body text keywords (up to 2)
                body_text = re.sub(r"<[^>]+>", "", sec.group(1)).strip()
                body_words = [w.lower() for w in body_text.split()
                              if len(w) > 3 and w.lower() not in filler]
                keywords.extend(body_words[:2])

        # Extract keywords from <execution_approaches> <approach> tags
        ea_match = re.search(r'<execution_approaches[^>]*>(.*?)</execution_approaches>', skill_content, re.DOTALL)
        if ea_match:
            for approach in re.finditer(r'<approach[^>]*name="([^"]+)"[^>]*>(.*?)</approach>', ea_match.group(1), re.DOTALL):
                # Approach name keywords
                name_words = [w.lower() for w in approach.group(1).split() if len(w) > 3 and w.lower() not in filler]
                keywords.extend(name_words[:2])
                # Approach description keywords
                desc_text = re.sub(r'<[^>]+>', '', approach.group(2)).strip()
                desc_words = [w.lower() for w in desc_text.split() if len(w) > 3 and w.lower() not in filler]
                keywords.extend(desc_words[:2])

        return keywords[:20]

    # Markdown: extract keywords from headings and step descriptions
    keywords = []
    filler = {"the", "this", "that", "with", "from", "for", "and",
              "but", "not", "all", "can", "will", "may", "should",
              "must", "also", "then", "than", "been", "being", "have"}

    # Get heading words as keywords
    headings = re.findall(r"^#{1,3}\s+(.+)$", skill_content, re.MULTILINE)
    for h in headings:
        words = [w.lower() for w in h.split() if len(w) > 3 and w.lower() not in filler]
        keywords.extend(words[:2])

    # Get distinctive words from numbered steps and key phrases
    body_start = skill_content.find("---", skill_content.find("---") + 3)
    body = skill_content[body_start + 3:] if body_start >= 0 else skill_content

    for line in body.split("\n"):
        stripped = line.strip()
        if stripped and (
            (stripped[0].isdigit() and "." in stripped[:4])
            or stripped.lower().startswith("step")
            or stripped.lower().startswith("never")
            or stripped.lower().startswith("avoid")
            or stripped.lower().startswith("always")
        ):
            words = [w.lower() for w in stripped.split() if len(w) > 3 and w.lower() not in filler]
            keywords.extend(words[:2])

    return keywords[:20]


def score_output(output: str, skill_content: str, skill_name: str) -> RubricScore:
    """Score an output on the instruction adherence rubric.

    Uses heuristic scoring based on the skill content and the output.
    Each dimension is scored 0-1 based on observable criteria.
    """
    frontmatter = _parse_frontmatter(skill_content)
    procedure_steps = _extract_procedure_steps(skill_content)
    key_sections = _extract_key_sections(skill_content)
    completeness_keywords = _extract_completeness_keywords(skill_content)

    # --- 1. Procedure adherence (0-1) ---
    if procedure_steps:
        # Check how many procedure steps are referenced in the output.
        # For short steps (Markdown), use first 5 words as keywords.
        # For long steps (XML with body content), use first line as title keywords.
        steps_mentioned = 0
        for step in procedure_steps:
            # Extract key words from step title (first line if multi-line)
            step_first_line = step.split("\n")[0].strip()
            # Take up to 5 meaningful words (skip very short ones like "a", "the")
            key_words = [
                w for w in step_first_line.lower().split()
                if len(w) > 2  # skip short filler words
            ][:5]
            if key_words and any(word in output.lower() for word in key_words):
                steps_mentioned += 1
        procedure_adherence = min(1.0, steps_mentioned / max(len(procedure_steps), 1))
    else:
        # No explicit procedures — score based on general task completion signals
        procedure_adherence = 0.5 if len(output) > 100 else 0.2

    # --- 2. Constraint respect (0-1) ---
    # Detect constraints in both Markdown and XML formats
    constraint_keywords = [
        "never", "must not", "should not", "avoid",
        "do not", "block", "critical", "forbidden",
    ]
    has_constraints = (
        any(kw in skill_content.lower() for kw in constraint_keywords)
        or "<strict_constraints>" in skill_content
        or "<system_constraint>" in skill_content
    )

    if has_constraints:
        constraint_acknowledged = any(
            kw in output.lower()
            for kw in [
                "constraint", "never", "must not", "avoid",
                "security", "safety", "careful", "forbidden",
                "anti_pattern", "strict", "hitl",
            ]
        )
        dangerous_keywords = ["rm -rf", "DROP", "DELETE", "TRUNCATE", "sudo", "CASCADE"]
        violations = sum(1 for kw in dangerous_keywords if kw in output)
        constraint_respect = max(
            0.0, min(1.0, constraint_acknowledged * 0.7 + (1.0 - violations * 0.3))
        )
    else:
        constraint_respect = 0.7  # neutral — no constraints to check

    # --- 3. Format compliance (0-1) ---
    # Detect structured format in both Markdown and XML reference content
    is_xml_ref = _is_xml_format(skill_content)
    if is_xml_ref:
        # XML format: check if output references the XML structure or content
        has_sections = any(
            tag in skill_content
            for tag in ["<intent>", "<execution_steps>", "<examples>"]
        )
    else:
        has_sections = any(
            section in skill_content
            for section in ["## Description", "## Procedures", "## Examples"]
        )

    if has_sections:
        # Check if output has any structured elements (Markdown or XML)
        output_has_structure = any(
            marker in output for marker in [
                "##", "# ", "1.", "Step", "**", "-",  # Markdown markers
                "<step", "<intent", "<constraint", "<procedure",  # XML markers
            ]
        )
        format_compliance = 0.8 if output_has_structure else 0.3
    else:
        format_compliance = 0.5

    # --- 4. Completeness (0-1) ---
    # Use content-relevant keywords instead of abstract section names,
    # because LLM outputs naturally reference topic words (e.g., "site explorer")
    # rather than meta-names (e.g., "procedures").
    if completeness_keywords:
        keywords_covered = sum(
            1 for kw in completeness_keywords if kw in output.lower()
        )
        completeness = min(1.0, keywords_covered / max(len(completeness_keywords), 1))
    elif key_sections:
        # Fallback: check abstract section names (works for Markdown format)
        sections_covered = sum(1 for s in key_sections if s in output.lower())
        completeness = min(1.0, sections_covered / max(len(key_sections), 1))
    else:
        completeness = 0.5 if len(output) > 200 else 0.3

    # --- 5. Security awareness (0-1) ---
    hitl_required = frontmatter.get("hitl_required", "false").lower() == "true"
    security_level = frontmatter.get("security_level", "").lower()
    has_security = hitl_required or security_level in ["high", "critical"]

    if has_security:
        security_acknowledged = any(
            kw in output.lower()
            for kw in [
                "approval", "confirm", "human", "hitl",
                "review", "security", "careful", "critical",
            ]
        )
        security_awareness = 0.9 if security_acknowledged else 0.2
    else:
        security_awareness = 0.7  # neutral — no security requirements

    # --- Overall score (weighted average) ---
    overall = (
        procedure_adherence * 0.30
        + constraint_respect * 0.25
        + format_compliance * 0.15
        + completeness * 0.20
        + security_awareness * 0.10
    )

    return RubricScore(
        procedure_adherence=round(procedure_adherence, 3),
        constraint_respect=round(constraint_respect, 3),
        format_compliance=round(format_compliance, 3),
        completeness=round(completeness, 3),
        security_awareness=round(security_awareness, 3),
        overall=round(overall, 3),
    )


# ============================================================================
# Experiment Runner
# ============================================================================

def run_experiment(
    skills: List[Dict[str, Any]],
    platform: str = "claude",
    timeout: int = 120,
    compiled_dir: str = "experiments/compiled",
    model: str = "glm-5.1",
    mode: str = "api",
    api_type: str = "openai",
) -> List[ComparisonResult]:
    """Run the full comparative experiment: original vs compiled."""
    comparisons: List[ComparisonResult] = []

    print(f"\n🧪 NSC Instruction Adherence Evaluation")
    print(f"   Platform: {platform}")
    print(f"   Model: {model}")
    print(f"   Mode: {mode}")
    print(f"   Skills: {len(skills)}")
    print(f"   Timeout: {timeout}s per task")
    print()

    for i, skill in enumerate(skills):
        skill_name = skill["name"]
        skill_content = skill["content"]
        print(f"[{i+1}/{len(skills)}] Evaluating: {skill_name}")

        # --- Condition A: Original SKILL.md ---
        print(f"  → Running original condition...")
        prompt_original = (
            f"You are an AI agent. Follow this skill exactly:\n\n"
            f"{skill_content}\n\n"
            f"Now execute this skill. Describe what you would do step by step, "
            f"respecting all constraints and security requirements."
        )

        start_time = time.time()
        result_a = run_llm_query(prompt_original, model=model, mode=mode, api_type=api_type, timeout=timeout)
        duration_a = time.time() - start_time

        output_a = result_a.get("output", "")
        rubric_a = score_output(output_a, skill_content, skill_name)

        if result_a.get("error"):
            print(f"    ⚠ Error: {result_a['error'][:200]}")

        print(f"    Score: {rubric_a.overall:.3f} (duration: {duration_a:.1f}s)")

        # --- Compile the skill ---
        print(f"  → Compiling skill for {platform}...")
        compiled_content = compile_skill(
            skill["path"], platform=platform, output_dir=compiled_dir,
        )

        if compiled_content is None:
            print(f"  ⚠ Compilation failed for {skill_name}, skipping from comparison")
            # Skip failed compilations entirely rather than scoring them as 0.
            # This prevents compilation failures from distorting aggregate statistics.
            continue

        # --- Condition B: Compiled platform-native format ---
        print(f"  → Running compiled condition...")
        prompt_compiled = (
            f"You are an AI agent. Follow this compiled skill exactly:\n\n"
            f"{compiled_content}\n\n"
            f"Now execute this skill. Describe what you would do step by step, "
            f"respecting all constraints and security requirements."
        )

        start_time = time.time()
        result_b = run_llm_query(prompt_compiled, model=model, mode=mode, api_type=api_type, timeout=timeout)
        duration_b = time.time() - start_time

        output_b = result_b.get("output", "")
        # Use compiled content as the scoring reference for Condition B,
        # not the original SKILL.md. This ensures fair evaluation:
        # "Did the LLM follow the instructions it actually received?"
        rubric_b = score_output(output_b, compiled_content, skill_name)

        if result_b.get("error"):
            print(f"    ⚠ Error: {result_b['error'][:200]}")

        print(f"    Score: {rubric_b.overall:.3f} (duration: {duration_b:.1f}s)")

        # --- Compare ---
        diff_overall = rubric_b.overall - rubric_a.overall
        orig_chars = len(skill_content)
        comp_chars = len(compiled_content)
        info_ratio = round(comp_chars / max(orig_chars, 1), 3)
        comparison = ComparisonResult(
            skill_name=skill_name,
            original_score=rubric_a,
            compiled_score=rubric_b,
            score_diff=RubricScore(
                procedure_adherence=round(rubric_b.procedure_adherence - rubric_a.procedure_adherence, 3),
                constraint_respect=round(rubric_b.constraint_respect - rubric_a.constraint_respect, 3),
                format_compliance=round(rubric_b.format_compliance - rubric_a.format_compliance, 3),
                completeness=round(rubric_b.completeness - rubric_a.completeness, 3),
                security_awareness=round(rubric_b.security_awareness - rubric_a.security_awareness, 3),
                overall=round(diff_overall, 3),
            ),
            significant_improvement=diff_overall > 0.1,
            original_char_count=orig_chars,
            compiled_char_count=comp_chars,
            information_ratio=info_ratio,
        )
        comparisons.append(comparison)

        direction = "📈 IMPROVED" if diff_overall > 0 else "📉 NO IMPROVEMENT"
        print(f"  ✅ Diff: {diff_overall:+.3f} {direction}")
        print()

    return comparisons


# ============================================================================
# Report Generation
# ============================================================================

def generate_report(comparisons: List[ComparisonResult], output_file: Path, model: str = "glm-5.1", mode: str = "api", api_type: str = "openai") -> None:
    """Generate a markdown report from comparison results."""

    total = len(comparisons)
    avg_original = sum(c.original_score.overall for c in comparisons) / max(total, 1)
    avg_compiled = sum(c.compiled_score.overall for c in comparisons) / max(total, 1)
    avg_diff = avg_compiled - avg_original
    improved = sum(1 for c in comparisons if c.significant_improvement)

    # Per-dimension averages
    dim_avgs: Dict[str, Dict[str, float]] = {}
    for dim in [
        "procedure_adherence", "constraint_respect",
        "format_compliance", "completeness", "security_awareness",
    ]:
        orig_avg = sum(getattr(c.original_score, dim) for c in comparisons) / max(total, 1)
        comp_avg = sum(getattr(c.compiled_score, dim) for c in comparisons) / max(total, 1)
        dim_avgs[dim] = {
            "original": round(orig_avg, 3),
            "compiled": round(comp_avg, 3),
            "diff": round(comp_avg - orig_avg, 3),
        }

    report = f"""# NSC Instruction Adherence Evaluation Report

**Generated**: {datetime.now().isoformat()}
**Skills evaluated**: {total}
**Platform**: Claude (NSC compilation target)
**Model**: {model}
**Mode**: {mode} ({api_type})

## Executive Summary

This experiment measures **instruction adherence accuracy** — whether compiled
platform-native formats improve LLM instruction following compared to raw SKILL.md.

| Metric | Original (SKILL.md) | Compiled (Platform-native) | Difference |
|--------|--------------------|---------------------------|------------|
| Overall Score | {avg_original:.3f} | {avg_compiled:.3f} | {avg_diff:+.3f} |
| Skills Improved | | | {improved}/{total} ({improved/max(total,1)*100:.0f}%) |

## Per-Dimension Results

| Dimension | Original | Compiled | Diff |
|-----------|----------|----------|------|
| Procedure Adherence | {dim_avgs['procedure_adherence']['original']} | {dim_avgs['procedure_adherence']['compiled']} | {dim_avgs['procedure_adherence']['diff']:+} |
| Constraint Respect | {dim_avgs['constraint_respect']['original']} | {dim_avgs['constraint_respect']['compiled']} | {dim_avgs['constraint_respect']['diff']:+} |
| Format Compliance | {dim_avgs['format_compliance']['original']} | {dim_avgs['format_compliance']['compiled']} | {dim_avgs['format_compliance']['diff']:+} |
| Completeness | {dim_avgs['completeness']['original']} | {dim_avgs['completeness']['compiled']} | {dim_avgs['completeness']['diff']:+} |
| Security Awareness | {dim_avgs['security_awareness']['original']} | {dim_avgs['security_awareness']['compiled']} | {dim_avgs['security_awareness']['diff']:+} |

## Per-Skill Results

| Skill | Original | Compiled | Diff | Improved? | Info Ratio |
|-------|----------|----------|------|-----------|------------|
"""

    for c in comparisons:
        improved_marker = "✅" if c.significant_improvement else "❌"
        report += (
            f"| {c.skill_name} "
            f"| {c.original_score.overall:.3f} "
            f"| {c.compiled_score.overall:.3f} "
            f"| {c.score_diff.overall:+.3f} "
            f"| {improved_marker} "
            f"| {c.information_ratio:.2f} |\n"
        )

    # Information preservation summary
    avg_info_ratio = sum(c.information_ratio for c in comparisons) / max(total, 1)

    report += f"""
## Information Preservation

| Metric | Value |
|--------|-------|
| Avg. Original Char Count | {sum(c.original_char_count for c in comparisons) / max(total, 1):.0f} |
| Avg. Compiled Char Count | {sum(c.compiled_char_count for c in comparisons) / max(total, 1):.0f} |
| Avg. Information Ratio | {avg_info_ratio:.2f} |

Information Ratio = compiled_chars / original_chars. Values near 1.0 indicate
good information preservation; values < 0.5 indicate significant information loss.

## Methodology

### Experimental Design

- **Condition A (Original)**: LLM receives raw SKILL.md content, scored against SKILL.md
- **Condition B (Compiled)**: LLM receives NSC-compiled platform-native format (Claude XML), scored against compiled content
- **Task**: "Follow this skill exactly, describe what you would do step by step"
- **Scoring**: 5-dimension rubric (Procedure, Constraint, Format, Completeness, Security)
- **Fair evaluation**: Each condition is scored against the content it actually received,
  measuring "Did the LLM follow the instructions it was given?" rather than
  "Did the LLM follow instructions it never received?"

### Scoring Rubric

Each dimension scored 0-1:
- **Procedure Adherence (30% weight)**: Did the LLM follow all procedure steps?
- **Constraint Respect (25% weight)**: Did the LLM respect security constraints?
- **Format Compliance (15% weight)**: Was output in expected format structure?
- **Completeness (20% weight)**: Did output cover all required sections?
- **Security Awareness (10% weight)**: Did LLM acknowledge HITL/security requirements?

### Information Preservation

Information Ratio (compiled_chars / original_chars) quantifies how much
of the original content is preserved in the compiled format. This serves
as a proxy for the LLM's available context in Condition B.

### Academic Basis

This evaluation design follows the format sensitivity research documented in
"高级提示词工程格式与智能体技能架构" — measuring whether format-specific
compilation improves LLM instruction adherence, not execution speed.

## Limitations

1. Heuristic scoring may not capture all nuances of instruction adherence
2. Claude Code headless mode may produce different outputs than interactive mode
3. Small sample size may limit statistical significance
4. Single-platform evaluation (Claude only) — multi-platform evaluation recommended
5. Failed compilations are excluded from statistics (not scored as 0)
"""

    output_file.write_text(report, encoding="utf-8")
    print(f"\n📄 Report saved to: {output_file}")


def save_raw_results(comparisons: List[ComparisonResult], output_file: Path) -> None:
    """Save raw comparison data as JSON for further analysis."""
    data = {
        "timestamp": datetime.now().isoformat(),
        "comparisons": [asdict(c) for c in comparisons],
    }
    output_file.write_text(
        json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8",
    )
    print(f"📊 Raw data saved to: {output_file}")


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="NSC Instruction Adherence Evaluation",
    )
    parser.add_argument(
        "--skills-dir", type=str,
        default="tests/fixtures/real_corpus",
        help="Directory containing skill .md files",
    )
    parser.add_argument(
        "--platform", type=str, default="claude",
        choices=["claude", "codex", "gemini", "kimi"],
        help="Target platform for compilation",
    )
    parser.add_argument(
        "--limit", type=int, default=None,
        help="Limit number of skills to evaluate",
    )
    parser.add_argument(
        "--all", action="store_true",
        help="Evaluate all skills (no limit)",
    )
    parser.add_argument(
        "--timeout", type=int, default=120,
        help="Timeout per Claude Code execution (seconds)",
    )
    parser.add_argument(
        "--output-dir", type=str, default="experiments/eval_results",
        help="Output directory for results",
    )
    parser.add_argument(
        "--compiled-dir", type=str, default="experiments/compiled",
        help="Output directory for compiled skills",
    )
    parser.add_argument(
        "--model", type=str, default="glm-5.1",
        help="Model ID to use (e.g., glm-5.1, deepseek-chat, claude-sonnet-4-20250514)",
    )
    parser.add_argument(
        "--mode", type=str, default="api", choices=["api", "cli"],
        help="Execution mode: 'api' (SDK direct) or 'cli' (Claude Code CLI)",
    )
    parser.add_argument(
        "--api-type", type=str, default="openai", choices=["openai", "anthropic"],
        help="API type for api mode: 'openai' (OpenAI-compatible proxy) or 'anthropic' (direct Anthropic)",
    )

    args = parser.parse_args()

    limit = None if args.all else args.limit
    skills_dir = Path(args.skills_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load skills
    skills = load_skills(skills_dir, limit=limit)

    if not skills:
        print(f"❌ No skills found in {skills_dir}")
        sys.exit(1)

    print(f"📚 Loaded {len(skills)} skills from {skills_dir}")

    # Run experiment
    comparisons = run_experiment(
        skills,
        platform=args.platform,
        timeout=args.timeout,
        compiled_dir=args.compiled_dir,
        model=args.model,
        mode=args.mode,
        api_type=args.api_type,
    )

    if not comparisons:
        print("❌ No comparisons generated")
        sys.exit(1)

    # Generate reports
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    report_file = output_dir / f"EVAL-{timestamp}_report.md"
    raw_file = output_dir / f"EVAL-{timestamp}_raw.json"

    generate_report(comparisons, report_file, model=args.model, mode=args.mode, api_type=args.api_type)
    save_raw_results(comparisons, raw_file)

    # Print summary
    avg_diff = sum(c.score_diff.overall for c in comparisons) / max(len(comparisons), 1)
    improved = sum(1 for c in comparisons if c.significant_improvement)

    print(f"\n{'='*60}")
    print(f"📊 SUMMARY")
    print(f"{'='*60}")
    print(f"Skills evaluated: {len(comparisons)}")
    print(f"Average improvement: {avg_diff:+.3f}")
    print(f"Skills with significant improvement (>0.1): {improved}/{len(comparisons)}")

    if avg_diff > 0.05:
        print(f"✅ Compiled formats show measurable improvement in instruction adherence")
    elif avg_diff > 0:
        print(f"⚠️ Compiled formats show slight improvement (may not be significant)")
    else:
        print(f"❌ No improvement detected — further investigation needed")

    print(f"{'='*60}")


if __name__ == "__main__":
    main()