#!/usr/bin/env python3
"""
Fetch Real Skill Datasets from GitHub

This script downloads real SKILL.md files from authoritative GitHub repositories
and stores them in tests/fixtures/real_corpus/ for testing purposes.

Data Sources:
- https://github.com/ComposioHQ/awesome-claude-skills
- https://github.com/heilcheng/awesome-agent-skills
- https://github.com/anthropics/skills

Usage:
    python scripts/fetch_real_skills.py [--batch BATCH_ID] [--limit LIMIT]
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import List, Optional
from datetime import datetime


# ============================================================================
# Configuration
# ============================================================================

PROJECT_ROOT = Path(__file__).parent.parent
FIXTURES_DIR = PROJECT_ROOT / "tests" / "fixtures" / "real_corpus"
TEMP_DIR = PROJECT_ROOT / "target" / "skill_fetch"
MANIFEST_FILE = FIXTURES_DIR / "manifest.json"

# GitHub repositories to fetch skills from
REPOSITORIES = {
    "anthropics": {
        "url": "https://github.com/anthropics/skills.git",
        "skill_paths": ["skills/docx", "skills/pdf", "skills/pptx", "skills/xlsx"],
        "category": "official",
        "description": "Official Anthropic document processing skills"
    },
    "awesome-claude-skills": {
        "url": "https://github.com/ComposioHQ/awesome-claude-skills.git",
        "skill_paths": [
            # Development & Code Tools (root level)
            "skill-creator",
            "mcp-builder",
            "changelog-generator",
            "webapp-testing",
            "langsmith-fetch",
            "connect",
            # Data & Analysis
            "lead-research-assistant",
            "meeting-insights-analyzer",
            # Business & Marketing
            "brand-guidelines",
            "competitive-ads-extractor",
            "domain-name-brainstormer",
            "internal-comms",
            # Creative & Media
            "canvas-design",
            "image-enhancer",
            "slack-gif-creator",
            "theme-factory",
            "video-downloader",
            # Productivity & Organization
            "file-organizer",
            "invoice-organizer",
            "raffle-winner-picker",
            "tailored-resume-generator",
        ],
        "category": "community",
        "description": "Composio curated Claude skills collection"
    },
    "composio-skills": {
        "url": "https://github.com/ComposioHQ/awesome-claude-skills.git",
        "skill_paths": [
            # SaaS Automation (in composio-skills subdirectory) - representative selection
            "composio-skills/slackbot-automation",
            "composio-skills/discordbot-automation",
            "composio-skills/salesforce-marketing-cloud-automation",
            "composio-skills/salesforce-service-cloud-automation",
            "composio-skills/anthropic-administrator-automation",
            "composio-skills/amazon-automation",
            "composio-skills/active-campaign-automation",
            "composio-skills/ahrefs-automation",
            "composio-skills/algolia-automation",
            "composio-skills/api-ninjas-automation",
        ],
        "category": "saas",
        "description": "Composio SaaS automation skills"
    },
    "awesome-agent-skills": {
        "url": "https://github.com/heilcheng/awesome-agent-skills.git",
        "skill_paths": [],  # This is a curated list, skills are in external repos
        "category": "index",
        "description": "Curated list of agent skills from multiple sources"
    },
}

# Batch definitions for progressive testing
BATCHES = {
    1: {
        "name": "Official Anthropic Skills",
        "sources": ["anthropics"],
        "skills": ["docx", "pdf", "pptx", "xlsx"],
        "complexity": "medium",
    },
    2: {
        "name": "Development Tools",
        "sources": ["awesome-claude-skills"],
        "skills": [
            "skill-creator",
            "mcp-builder",
            "changelog-generator",
            "webapp-testing",
            "langsmith-fetch",
            "connect",
        ],
        "complexity": "medium",
    },
    3: {
        "name": "Business & Creative",
        "sources": ["awesome-claude-skills"],
        "skills": [
            "brand-guidelines",
            "competitive-ads-extractor",
            "domain-name-brainstormer",
            "canvas-design",
            "image-enhancer",
            "theme-factory",
        ],
        "complexity": "simple",
    },
    4: {
        "name": "Productivity Tools",
        "sources": ["awesome-claude-skills"],
        "skills": [
            "file-organizer",
            "invoice-organizer",
            "raffle-winner-picker",
            "tailored-resume-generator",
            "lead-research-assistant",
            "meeting-insights-analyzer",
        ],
        "complexity": "simple",
    },
    5: {
        "name": "SaaS Automation - Communication",
        "sources": ["composio-skills"],
        "skills": [
            "slackbot-automation",
            "discordbot-automation",
        ],
        "complexity": "medium",
    },
    6: {
        "name": "SaaS Automation - Enterprise",
        "sources": ["composio-skills"],
        "skills": [
            "salesforce-marketing-cloud-automation",
            "salesforce-service-cloud-automation",
            "anthropic-administrator-automation",
        ],
        "complexity": "complex",
    },
    7: {
        "name": "SaaS Automation - APIs & Tools",
        "sources": ["composio-skills"],
        "skills": [
            "amazon-automation",
            "active-campaign-automation",
            "ahrefs-automation",
            "algolia-automation",
            "api-ninjas-automation",
        ],
        "complexity": "medium",
    },
}


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class SkillMeta:
    """Metadata for a fetched skill file."""
    name: str
    source: str
    source_url: str
    category: str
    batch_id: int
    complexity: str
    file_path: str
    fetched_at: str
    file_size: int
    has_frontmatter: bool
    error: Optional[str] = None


@dataclass
class FetchResult:
    """Result of fetching skills from a repository."""
    repo_name: str
    repo_url: str
    success: bool
    skills_fetched: int
    skills_failed: int
    errors: List[str]
    duration_seconds: float


# ============================================================================
# Helper Functions
# ============================================================================

def log_info(msg: str) -> None:
    """Print info message."""
    print(f"[INFO] {msg}")


def log_warn(msg: str) -> None:
    """Print warning message."""
    print(f"[WARN] {msg}")


def log_error(msg: str) -> None:
    """Print error message."""
    print(f"[ERROR] {msg}")


def run_command(cmd: List[str], cwd: Optional[Path] = None) -> subprocess.CompletedProcess:
    """Run a shell command and return result."""
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=300  # 5 minute timeout
    )


def clone_repo(repo_url: str, target_dir: Path) -> bool:
    """Clone a GitHub repository."""
    if target_dir.exists():
        log_info(f"Repository already exists at {target_dir}, updating...")
        result = run_command(["git", "pull"], cwd=target_dir)
        if result.returncode != 0:
            log_warn(f"Failed to update repo, re-cloning...")
            shutil.rmtree(target_dir)
            result = run_command(["git", "clone", "--depth=1", repo_url, str(target_dir)])
            return result.returncode == 0
        return True
    else:
        log_info(f"Cloning {repo_url}...")
        result = run_command(["git", "clone", "--depth=1", repo_url, str(target_dir)])
        return result.returncode == 0


def find_skill_files(repo_dir: Path, skill_paths: List[str]) -> List[Path]:
    """Find SKILL.md files in a repository."""
    skill_files = []
    
    for skill_path in skill_paths:
        # Try direct path
        direct_path = repo_dir / skill_path / "SKILL.md"
        if direct_path.exists():
            skill_files.append(direct_path)
            continue
        
        # Try lowercase
        lower_path = repo_dir / skill_path.lower() / "SKILL.md"
        if lower_path.exists():
            skill_files.append(lower_path)
            continue
        
        # Try with -skill suffix
        suffix_path = repo_dir / f"{skill_path}-skill" / "SKILL.md"
        if suffix_path.exists():
            skill_files.append(suffix_path)
            continue
        
        # Search for any SKILL.md in subdirectories
        for found in repo_dir.rglob("SKILL.md"):
            if skill_path.lower() in found.parent.name.lower():
                skill_files.append(found)
                break
    
    return skill_files


def check_frontmatter(content: str) -> bool:
    """Check if content has YAML frontmatter."""
    return content.strip().startswith("---")


def copy_skill_file(
    skill_file: Path,
    dest_dir: Path,
    source: str,
    source_url: str,
    category: str,
    batch_id: int,
    complexity: str
) -> Optional[SkillMeta]:
    """Copy a skill file to the fixtures directory and create metadata."""
    try:
        # Read content
        content = skill_file.read_text(encoding="utf-8")
        
        # Determine destination filename
        skill_name = skill_file.parent.name
        dest_file = dest_dir / f"{skill_name}.md"
        
        # Copy file
        shutil.copy2(skill_file, dest_file)
        
        # Create metadata
        meta = SkillMeta(
            name=skill_name,
            source=source,
            source_url=source_url,
            category=category,
            batch_id=batch_id,
            complexity=complexity,
            file_path=str(dest_file.relative_to(PROJECT_ROOT)),
            fetched_at=datetime.now().isoformat(),
            file_size=dest_file.stat().st_size,
            has_frontmatter=check_frontmatter(content),
        )
        
        log_info(f"  ✓ Copied {skill_name} ({meta.file_size} bytes)")
        return meta
        
    except Exception as e:
        log_error(f"  ✗ Failed to copy {skill_file}: {e}")
        return SkillMeta(
            name=skill_file.parent.name,
            source=source,
            source_url=source_url,
            category=category,
            batch_id=batch_id,
            complexity=complexity,
            file_path="",
            fetched_at=datetime.now().isoformat(),
            file_size=0,
            has_frontmatter=False,
            error=str(e),
        )


# ============================================================================
# Main Fetch Functions
# ============================================================================

def fetch_repository(
    repo_name: str,
    repo_info: dict,
    temp_dir: Path,
    dest_dir: Path,
    batch_id: int = 0,
    limit: Optional[int] = None
) -> FetchResult:
    """Fetch skills from a single repository."""
    start_time = time.time()
    repo_dir = temp_dir / repo_name
    
    # Clone repository
    if not clone_repo(repo_info["url"], repo_dir):
        return FetchResult(
            repo_name=repo_name,
            repo_url=repo_info["url"],
            success=False,
            skills_fetched=0,
            skills_failed=0,
            errors=["Failed to clone repository"],
            duration_seconds=time.time() - start_time,
        )
    
    # Find skill files
    skill_files = find_skill_files(repo_dir, repo_info["skill_paths"])
    
    # If no specific paths, search all SKILL.md files
    if not skill_files and repo_info["skill_paths"]:
        log_warn(f"No skills found in specified paths for {repo_name}")
    
    # Search for all SKILL.md if paths are empty
    if not repo_info["skill_paths"]:
        skill_files = list(repo_dir.rglob("SKILL.md"))
        log_info(f"Found {len(skill_files)} SKILL.md files in {repo_name}")
    
    # Apply limit
    if limit and len(skill_files) > limit:
        skill_files = skill_files[:limit]
        log_info(f"Limiting to {limit} files")
    
    # Copy skill files
    skills_fetched = 0
    skills_failed = 0
    errors = []
    skill_metadata = []
    
    for skill_file in skill_files:
        meta = copy_skill_file(
            skill_file,
            dest_dir,
            repo_name,
            repo_info["url"],
            repo_info["category"],
            batch_id,
            "medium"  # Default complexity
        )
        if meta:
            skill_metadata.append(meta)
            if meta.error:
                skills_failed += 1
                errors.append(meta.error)
            else:
                skills_fetched += 1
    
    # Save individual metadata
    for meta in skill_metadata:
        meta_file = dest_dir / f"{meta.name}.meta.json"
        with open(meta_file, "w", encoding="utf-8") as f:
            json.dump(asdict(meta), f, indent=2)
    
    return FetchResult(
        repo_name=repo_name,
        repo_url=repo_info["url"],
        success=True,
        skills_fetched=skills_fetched,
        skills_failed=skills_failed,
        errors=errors,
        duration_seconds=time.time() - start_time,
    )


def fetch_batch(
    batch_id: int,
    temp_dir: Path,
    dest_dir: Path
) -> List[SkillMeta]:
    """Fetch skills for a specific batch."""
    if batch_id not in BATCHES:
        log_error(f"Invalid batch ID: {batch_id}")
        return []
    
    batch = BATCHES[batch_id]
    log_info(f"Fetching Batch {batch_id}: {batch['name']}")
    
    all_metadata = []
    
    for source in batch["sources"]:
        if source not in REPOSITORIES:
            log_warn(f"Unknown source: {source}")
            continue
        
        repo_info = REPOSITORIES[source]
        repo_dir = temp_dir / source
        
        # Clone repository
        if not clone_repo(repo_info["url"], repo_dir):
            log_error(f"Failed to clone {source}")
            continue
        
        # Fetch specified skills for this batch
        for skill_name in batch["skills"]:
            skill_files = find_skill_files(repo_dir, [skill_name])
            
            if not skill_files:
                log_warn(f"  Skill not found: {skill_name}")
                continue
            
            for skill_file in skill_files:
                meta = copy_skill_file(
                    skill_file,
                    dest_dir,
                    source,
                    repo_info["url"],
                    repo_info["category"],
                    batch_id,
                    batch["complexity"]
                )
                if meta and not meta.error:
                    all_metadata.append(meta)
                    
                    # Save metadata
                    meta_file = dest_dir / f"{meta.name}.meta.json"
                    with open(meta_file, "w", encoding="utf-8") as f:
                        json.dump(asdict(meta), f, indent=2)
    
    log_info(f"Batch {batch_id} complete: {len(all_metadata)} skills fetched")
    return all_metadata


def update_manifest(metadata: List[SkillMeta]) -> None:
    """Update the manifest.json file with all skill metadata."""
    manifest = {
        "version": "1.0",
        "updated_at": datetime.now().isoformat(),
        "total_skills": len(metadata),
        "batches": {},
        "sources": {},
        "skills": [asdict(m) for m in metadata],
    }
    
    # Group by batch
    for meta in metadata:
        batch_id = str(meta.batch_id)
        if batch_id not in manifest["batches"]:
            manifest["batches"][batch_id] = []
        manifest["batches"][batch_id].append(meta.name)
    
    # Group by source
    for meta in metadata:
        if meta.source not in manifest["sources"]:
            manifest["sources"][meta.source] = []
        manifest["sources"][meta.source].append(meta.name)
    
    # Write manifest
    with open(MANIFEST_FILE, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    
    log_info(f"Manifest updated: {len(metadata)} skills registered")


# ============================================================================
# Main Entry Point
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Fetch real skill datasets from GitHub repositories"
    )
    parser.add_argument(
        "--batch",
        type=int,
        default=0,
        help="Fetch only a specific batch (1-8). Default: fetch all."
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit number of skills to fetch per repository"
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Clean existing fixtures before fetching"
    )
    
    args = parser.parse_args()
    
    # Create directories
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)
    TEMP_DIR.mkdir(parents=True, exist_ok=True)
    
    # Clean if requested
    if args.clean:
        log_info("Cleaning existing fixtures...")
        for f in FIXTURES_DIR.glob("*.md"):
            f.unlink()
        for f in FIXTURES_DIR.glob("*.meta.json"):
            f.unlink()
        if MANIFEST_FILE.exists():
            MANIFEST_FILE.unlink()
    
    all_metadata = []
    
    if args.batch > 0:
        # Fetch specific batch
        metadata = fetch_batch(args.batch, TEMP_DIR, FIXTURES_DIR)
        all_metadata.extend(metadata)
    else:
        # Fetch all batches progressively
        for batch_id in sorted(BATCHES.keys()):
            log_info(f"\n{'='*60}")
            log_info(f"Processing Batch {batch_id}: {BATCHES[batch_id]['name']}")
            log_info(f"{'='*60}")
            
            metadata = fetch_batch(batch_id, TEMP_DIR, FIXTURES_DIR)
            all_metadata.extend(metadata)
            
            # Pause between batches
            if batch_id < max(BATCHES.keys()):
                log_info("Pausing 5 seconds before next batch...")
                time.sleep(5)
    
    # Update manifest
    update_manifest(all_metadata)
    
    # Summary
    log_info(f"\n{'='*60}")
    log_info("FETCH COMPLETE")
    log_info(f"{'='*60}")
    log_info(f"Total skills fetched: {len(all_metadata)}")
    log_info(f"Output directory: {FIXTURES_DIR}")
    log_info(f"Manifest file: {MANIFEST_FILE}")
    
    # Print batch summary
    batch_counts = {}
    for meta in all_metadata:
        batch_id = meta.batch_id
        batch_counts[batch_id] = batch_counts.get(batch_id, 0) + 1
    
    for batch_id, count in sorted(batch_counts.items()):
        batch_name = BATCHES.get(batch_id, {}).get("name", "Unknown")
        log_info(f"  Batch {batch_id} ({batch_name}): {count} skills")


if __name__ == "__main__":
    main()