#!/usr/bin/env python3
"""
Academic Evaluation Runner for Nexa Skill Compiler

This script runs comparative experiments to evaluate the effectiveness of NSC
by comparing task execution with original skills vs compiled skills.

Usage:
    python scripts/academic_evaluation.py --mode original --limit 5
    python scripts/academic_evaluation.py --mode compiled --limit 5
    python scripts/academic_evaluation.py --mode compare --limit 10
    python scripts/academic_evaluation.py --all
"""

import argparse
import json
import os
import random
import shutil
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
EXPERIMENTS_DIR = PROJECT_ROOT / "experiments"
TASKS_DIR = EXPERIMENTS_DIR / "tasks"
RESULTS_DIR = EXPERIMENTS_DIR / "results"
LOGS_DIR = EXPERIMENTS_DIR / "logs"
ANALYSIS_DIR = EXPERIMENTS_DIR / "analysis"
REPORTS_DIR = EXPERIMENTS_DIR / "reports"

# Claude Code skill directories
CLAUDE_SKILLS_DIR = Path.home() / ".claude" / "skills"

# Original and compiled skill sources
ORIGINAL_SKILLS_DIR = PROJECT_ROOT / "tests" / "fixtures" / "real_corpus"
COMPILED_SKILLS_DIR = PROJECT_ROOT / "test_results" / "compiled"

# Default timeout for task execution (seconds)
DEFAULT_TIMEOUT = 300


class SkillType(Enum):
    ORIGINAL = "original"
    COMPILED = "compiled"


class TaskCategory(Enum):
    DOCUMENT = "document"
    CODE = "code"
    DATA = "data"
    SWE_BENCH = "swe_bench"


class TaskDifficulty(Enum):
    SIMPLE = "simple"
    MEDIUM = "medium"
    COMPLEX = "complex"


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class Task:
    """Represents a single evaluation task."""
    id: str
    name: str
    category: str
    difficulty: str
    skill: str
    prompt: str
    expected_output: str
    verification_type: str
    timeout_seconds: int = DEFAULT_TIMEOUT


@dataclass
class ExecutionResult:
    """Result of a single task execution."""
    task_id: str
    skill_type: str
    success: bool
    duration_seconds: float
    turns: int
    errors: List[str]
    output: str
    quality_score: float
    timestamp: str = field(default_factory=lambda: datetime.now().isoformat())


@dataclass
class ExperimentReport:
    """Complete experiment report."""
    experiment_id: str
    skill_type: str
    total_tasks: int
    success_count: int
    success_rate: float
    avg_duration_seconds: float
    avg_turns: float
    avg_quality_score: float
    results: List[Dict[str, Any]]
    started_at: str
    completed_at: str


# ============================================================================
# Task Definitions
# ============================================================================

def get_task_definitions() -> List[Task]:
    """Return all task definitions for the evaluation."""
    tasks = []
    
    # ========================================
    # Document Processing Tasks (15 tasks)
    # ========================================
    doc_tasks = [
        Task(
            id="DOC-001",
            name="Extract table from Word document to CSV",
            category="document",
            difficulty="simple",
            skill="docx",
            prompt="Read the Word document at experiments/tasks/input/sample.docx and extract the first table to a CSV file at experiments/tasks/output/table.csv",
            expected_output="CSV file with correct table data",
            verification_type="file_content"
        ),
        Task(
            id="DOC-002",
            name="Extract text from PDF",
            category="document",
            difficulty="simple",
            skill="pdf",
            prompt="Extract all text content from the PDF at experiments/tasks/input/sample.pdf and save it to experiments/tasks/output/text.txt",
            expected_output="Text file with extracted content",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-003",
            name="Create PowerPoint presentation",
            category="document",
            difficulty="medium",
            skill="pptx",
            prompt="Create a PowerPoint presentation at experiments/tasks/output/presentation.pptx with 3 slides: Title slide, Content slide with bullet points, and Conclusion slide",
            expected_output="PPTX file with 3 slides",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-004",
            name="Convert Excel to JSON",
            category="document",
            difficulty="medium",
            skill="xlsx",
            prompt="Read the Excel file at experiments/tasks/input/data.xlsx and convert the first sheet to JSON format, save to experiments/tasks/output/data.json",
            expected_output="JSON file with sheet data",
            verification_type="file_content"
        ),
        Task(
            id="DOC-005",
            name="Generate Excel report",
            category="document",
            difficulty="medium",
            skill="xlsx",
            prompt="Create an Excel file at experiments/tasks/output/report.xlsx with a summary sheet containing headers: Name, Value, Status and 3 rows of sample data",
            expected_output="Excel file with data",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-006",
            name="Merge multiple text files",
            category="document",
            difficulty="simple",
            skill="file-organizer",
            prompt="Merge all .txt files in experiments/tasks/input/txt/ into a single file at experiments/tasks/output/merged.txt",
            expected_output="Merged text file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-007",
            name="Organize files by extension",
            category="document",
            difficulty="simple",
            skill="file-organizer",
            prompt="Organize files in experiments/tasks/input/mixed/ by their extension into subdirectories",
            expected_output="Organized directory structure",
            verification_type="directory_structure"
        ),
        Task(
            id="DOC-008",
            name="Create markdown documentation",
            category="document",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create a markdown documentation file at experiments/tasks/output/README.md describing a simple Python function that calculates factorial",
            expected_output="Markdown file with documentation",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-009",
            name="Generate changelog",
            category="document",
            difficulty="simple",
            skill="changelog-generator",
            prompt="Generate a changelog from the git history of this repository and save it to experiments/tasks/output/CHANGELOG.md",
            expected_output="Changelog markdown file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-010",
            name="Create invoice summary",
            category="document",
            difficulty="medium",
            skill="invoice-organizer",
            prompt="Create a summary report of invoice data at experiments/tasks/output/invoice_summary.txt",
            expected_output="Invoice summary file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-011",
            name="Generate resume",
            category="document",
            difficulty="medium",
            skill="tailored-resume-generator",
            prompt="Generate a simple resume markdown file at experiments/tasks/output/resume.md for a software engineer position",
            expected_output="Resume markdown file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-012",
            name="Create brand guidelines doc",
            category="document",
            difficulty="simple",
            skill="brand-guidelines",
            prompt="Create a simple brand guidelines document at experiments/tasks/output/brand.md with color palette and typography sections",
            expected_output="Brand guidelines file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-013",
            name="Generate meeting analysis",
            category="document",
            difficulty="medium",
            skill="meeting-insights-analyzer",
            prompt="Create a meeting analysis template at experiments/tasks/output/meeting_analysis.md",
            expected_output="Meeting analysis file",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-014",
            name="Create lead research template",
            category="document",
            difficulty="medium",
            skill="lead-research-assistant",
            prompt="Create a lead research template at experiments/tasks/output/lead_template.md",
            expected_output="Lead research template",
            verification_type="file_exists"
        ),
        Task(
            id="DOC-015",
            name="Generate raffle report",
            category="document",
            difficulty="simple",
            skill="raffle-winner-picker",
            prompt="Create a raffle report template at experiments/tasks/output/raffle_report.md",
            expected_output="Raffle report file",
            verification_type="file_exists"
        ),
    ]
    tasks.extend(doc_tasks)
    
    # ========================================
    # Code Analysis Tasks (15 tasks)
    # ========================================
    code_tasks = [
        Task(
            id="CODE-001",
            name="Analyze Python code structure",
            category="code",
            difficulty="simple",
            skill="skill-creator",
            prompt="Analyze the Python file at experiments/tasks/input/sample.py and list all function names to experiments/tasks/output/functions.txt",
            expected_output="List of function names",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-002",
            name="Generate code documentation",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Generate docstrings for the Python file at experiments/tasks/input/sample.py and save the updated file to experiments/tasks/output/documented.py",
            expected_output="Python file with docstrings",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-003",
            name="Create unit test template",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create a unit test file at experiments/tasks/output/test_sample.py for the functions in experiments/tasks/input/sample.py",
            expected_output="Test file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-004",
            name="Generate MCP server template",
            category="code",
            difficulty="complex",
            skill="mcp-builder",
            prompt="Create a simple MCP server template at experiments/tasks/output/mcp_server.py that provides a hello world tool",
            expected_output="MCP server file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-005",
            name="Create webapp test script",
            category="code",
            difficulty="medium",
            skill="webapp-testing",
            prompt="Create a Playwright test script at experiments/tasks/output/test_webapp.py that navigates to example.com and checks the title",
            expected_output="Playwright test file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-006",
            name="Generate LangSmith fetch script",
            category="code",
            difficulty="medium",
            skill="langsmith-fetch",
            prompt="Create a script at experiments/tasks/output/langsmith_example.py that demonstrates how to fetch traces from LangSmith",
            expected_output="LangSmith script",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-007",
            name="Create connect app example",
            category="code",
            difficulty="simple",
            skill="connect",
            prompt="Create an example script at experiments/tasks/output/connect_example.py showing how to use the connect skill",
            expected_output="Connect example file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-008",
            name="Analyze code complexity",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Analyze the complexity of the code in experiments/tasks/input/sample.py and create a report at experiments/tasks/output/complexity.txt",
            expected_output="Complexity report",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-009",
            name="Generate API client code",
            category="code",
            difficulty="complex",
            skill="skill-creator",
            prompt="Create a simple REST API client at experiments/tasks/output/api_client.py with GET and POST methods",
            expected_output="API client file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-010",
            name="Create configuration file",
            category="code",
            difficulty="simple",
            skill="skill-creator",
            prompt="Create a JSON configuration file at experiments/tasks/output/config.json with database, api, and logging settings",
            expected_output="Config JSON file",
            verification_type="file_content"
        ),
        Task(
            id="CODE-011",
            name="Generate error handling code",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create a Python module at experiments/tasks/output/error_handler.py with custom exception classes and a retry decorator",
            expected_output="Error handler module",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-012",
            name="Create logging setup",
            category="code",
            difficulty="simple",
            skill="skill-creator",
            prompt="Create a logging configuration module at experiments/tasks/output/logger.py with file and console handlers",
            expected_output="Logger module",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-013",
            name="Generate data models",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create Pydantic data models at experiments/tasks/output/models.py for User, Product, and Order entities",
            expected_output="Models file",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-014",
            name="Create validation utilities",
            category="code",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create a validation utilities module at experiments/tasks/output/validators.py with email, phone, and URL validators",
            expected_output="Validators module",
            verification_type="file_exists"
        ),
        Task(
            id="CODE-015",
            name="Generate CLI skeleton",
            category="code",
            difficulty="complex",
            skill="skill-creator",
            prompt="Create a CLI application skeleton at experiments/tasks/output/cli.py using argparse with init, run, and status commands",
            expected_output="CLI skeleton file",
            verification_type="file_exists"
        ),
    ]
    tasks.extend(code_tasks)
    
    # ========================================
    # Data Processing Tasks (10 tasks)
    # ========================================
    data_tasks = [
        Task(
            id="DATA-001",
            name="Process CSV data",
            category="data",
            difficulty="simple",
            skill="file-organizer",
            prompt="Read the CSV file at experiments/tasks/input/data.csv and create a summary at experiments/tasks/output/csv_summary.txt with row count and column names",
            expected_output="CSV summary file",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-002",
            name="Convert JSON to YAML",
            category="data",
            difficulty="simple",
            skill="file-organizer",
            prompt="Convert the JSON file at experiments/tasks/input/config.json to YAML format and save to experiments/tasks/output/config.yaml",
            expected_output="YAML config file",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-003",
            name="Generate data statistics",
            category="data",
            difficulty="medium",
            skill="file-organizer",
            prompt="Calculate statistics (mean, median, std) for numeric columns in experiments/tasks/input/numbers.csv and save to experiments/tasks/output/stats.json",
            expected_output="Statistics JSON file",
            verification_type="file_content"
        ),
        Task(
            id="DATA-004",
            name="Create data visualization script",
            category="data",
            difficulty="medium",
            skill="canvas-design",
            prompt="Create a matplotlib script at experiments/tasks/output/visualize.py that generates a bar chart from data in experiments/tasks/input/chart_data.csv",
            expected_output="Visualization script",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-005",
            name="Generate sample dataset",
            category="data",
            difficulty="simple",
            skill="file-organizer",
            prompt="Generate a sample CSV dataset at experiments/tasks/output/sample_data.csv with 100 rows of random user data (id, name, email, age)",
            expected_output="Sample CSV file",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-006",
            name="Create data pipeline script",
            category="data",
            difficulty="complex",
            skill="file-organizer",
            prompt="Create a data pipeline script at experiments/tasks/output/pipeline.py that reads, transforms, and saves data",
            expected_output="Pipeline script",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-007",
            name="Generate SQL schema",
            category="data",
            difficulty="medium",
            skill="skill-creator",
            prompt="Create a SQL schema file at experiments/tasks/output/schema.sql with tables for users, orders, and products",
            expected_output="SQL schema file",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-008",
            name="Create data validation script",
            category="data",
            difficulty="medium",
            skill="file-organizer",
            prompt="Create a data validation script at experiments/tasks/output/validate.py that checks CSV files for missing values and outliers",
            expected_output="Validation script",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-009",
            name="Generate report template",
            category="data",
            difficulty="medium",
            skill="canvas-design",
            prompt="Create a report template at experiments/tasks/output/report_template.md with sections for executive summary, methodology, and results",
            expected_output="Report template",
            verification_type="file_exists"
        ),
        Task(
            id="DATA-010",
            name="Create ETL example",
            category="data",
            difficulty="complex",
            skill="file-organizer",
            prompt="Create an ETL example script at experiments/tasks/output/etl.py demonstrating extract, transform, load operations",
            expected_output="ETL script",
            verification_type="file_exists"
        ),
    ]
    tasks.extend(data_tasks)
    
    # ========================================
    # SWE-bench Style Tasks (10 tasks)
    # ========================================
    swe_tasks = [
        Task(
            id="SWE-001",
            name="Fix Python indentation bug",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Fix the indentation bug in experiments/tasks/input/buggy.py and save the fixed version to experiments/tasks/output/fixed.py",
            expected_output="Fixed Python file",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-002",
            name="Add error handling to function",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Add proper error handling to the function in experiments/tasks/input/no_error_handling.py and save to experiments/tasks/output/with_error_handling.py",
            expected_output="File with error handling",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-003",
            name="Refactor duplicate code",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Refactor the duplicate code in experiments/tasks/input/duplicates.py into reusable functions and save to experiments/tasks/output/refactored.py",
            expected_output="Refactored file",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-004",
            name="Add type hints to code",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Add type hints to the Python file at experiments/tasks/input/no_types.py and save to experiments/tasks/output/with_types.py",
            expected_output="File with type hints",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-005",
            name="Fix import order",
            category="swe_bench",
            difficulty="simple",
            skill="skill-creator",
            prompt="Fix the import order in experiments/tasks/input/messy_imports.py following PEP 8 and save to experiments/tasks/output/clean_imports.py",
            expected_output="File with clean imports",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-006",
            name="Add logging to module",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Add logging statements to experiments/tasks/input/no_logging.py and save to experiments/tasks/output/with_logging.py",
            expected_output="File with logging",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-007",
            name="Convert to async function",
            category="swe_bench",
            difficulty="complex",
            skill="skill-creator",
            prompt="Convert the synchronous function in experiments/tasks/input/sync_func.py to async and save to experiments/tasks/output/async_func.py",
            expected_output="Async function file",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-008",
            name="Add input validation",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Add input validation to the function in experiments/tasks/input/no_validation.py and save to experiments/tasks/output/with_validation.py",
            expected_output="File with validation",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-009",
            name="Implement missing function",
            category="swe_bench",
            difficulty="medium",
            skill="skill-creator",
            prompt="Implement the missing function indicated by TODO in experiments/tasks/input/todo.py and save to experiments/tasks/output/implemented.py",
            expected_output="Implemented function file",
            verification_type="file_exists"
        ),
        Task(
            id="SWE-010",
            name="Optimize slow code",
            category="swe_bench",
            difficulty="complex",
            skill="skill-creator",
            prompt="Optimize the slow code in experiments/tasks/input/slow.py and save the optimized version to experiments/tasks/output/fast.py",
            expected_output="Optimized file",
            verification_type="file_exists"
        ),
    ]
    tasks.extend(swe_tasks)
    
    return tasks


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


def ensure_directories() -> None:
    """Create necessary directories."""
    for d in [EXPERIMENTS_DIR, TASKS_DIR, RESULTS_DIR, LOGS_DIR, ANALYSIS_DIR, REPORTS_DIR]:
        d.mkdir(parents=True, exist_ok=True)
    
    # Create input/output directories
    (TASKS_DIR / "input").mkdir(parents=True, exist_ok=True)
    (TASKS_DIR / "output").mkdir(parents=True, exist_ok=True)


def deploy_skill(skill_name: str, skill_type: SkillType) -> bool:
    """Deploy a skill to Claude Code skills directory."""
    if skill_type == SkillType.ORIGINAL:
        src = ORIGINAL_SKILLS_DIR / f"{skill_name}.md"
    else:
        # Compiled skills have .md extension
        src = COMPILED_SKILLS_DIR / f"{skill_name}.md"
    
    if not src.exists():
        log_warn(f"Skill file not found: {src}")
        return False
    
    # Create skill directory
    skill_dir = CLAUDE_SKILLS_DIR / skill_name
    skill_dir.mkdir(parents=True, exist_ok=True)
    
    # Copy skill file
    dst = skill_dir / "SKILL.md"
    shutil.copy(src, dst)
    
    log_info(f"Deployed {skill_type.value} skill: {skill_name}")
    return True


def clear_skills() -> None:
    """Clear all skills from Claude Code skills directory."""
    if CLAUDE_SKILLS_DIR.exists():
        for skill_dir in CLAUDE_SKILLS_DIR.iterdir():
            if skill_dir.is_symlink():
                # Handle symbolic links by unlinking them
                os.unlink(skill_dir)
            elif skill_dir.is_dir():
                shutil.rmtree(skill_dir)
            elif skill_dir.is_file():
                os.remove(skill_dir)
    log_info("Cleared all skills from Claude Code")


# ============================================================================
# Claude Code Execution
# ============================================================================

def execute_with_claude_code(
    prompt: str,
    timeout: int = DEFAULT_TIMEOUT,
    output_format: str = "json"
) -> Dict[str, Any]:
    """Execute a task using Claude Code in headless mode."""
    cmd = [
        "claude",
        "-p", prompt,
        "--output-format", output_format,
    ]
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(PROJECT_ROOT)
        )
        
        if result.returncode == 0 and output_format == "json":
            try:
                return json.loads(result.stdout)
            except json.JSONDecodeError:
                return {
                    "success": False,
                    "error": "Failed to parse JSON output",
                    "raw_output": result.stdout[:500]
                }
        else:
            return {
                "success": result.returncode == 0,
                "output": result.stdout,
                "error": result.stderr if result.stderr else None
            }
            
    except subprocess.TimeoutExpired:
        return {
            "success": False,
            "error": f"Timeout after {timeout} seconds"
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }


def verify_output(result: Dict[str, Any], task: Task) -> tuple[bool, float]:
    """Verify the output of a task execution."""
    verification_type = task.verification_type
    output_dir = TASKS_DIR / "output"
    
    quality_score = 3.0  # Default score
    
    if verification_type == "file_exists":
        # Check if expected output file exists
        expected_file = output_dir / task.expected_output.split("/")[-1]
        if expected_file.exists():
            quality_score = 4.0
            return True, quality_score
        return False, 1.0
    
    elif verification_type == "file_content":
        # Check if file exists and has content
        expected_file = output_dir / task.expected_output.split("/")[-1]
        if expected_file.exists() and expected_file.stat().st_size > 0:
            quality_score = 4.5
            return True, quality_score
        return False, 1.0
    
    elif verification_type == "directory_structure":
        # Check if output directory has organized structure
        if output_dir.exists() and len(list(output_dir.iterdir())) > 0:
            quality_score = 3.5
            return True, quality_score
        return False, 1.0
    
    else:
        # Default: check if result indicates success
        if result.get("success", False):
            quality_score = 4.0
            return True, quality_score
        return False, 1.0


# ============================================================================
# Experiment Runner
# ============================================================================

class AcademicEvaluator:
    """Main evaluator class for running comparative experiments."""
    
    def __init__(self, skill_type: SkillType):
        self.skill_type = skill_type
        self.results: List[ExecutionResult] = []
        self.experiment_id = f"EXP-{datetime.now().strftime('%Y%m%d-%H%M%S')}"
        
    def run_task(self, task: Task) -> ExecutionResult:
        """Run a single task."""
        log_info(f"Running task {task.id}: {task.name}")
        
        # Deploy the required skill
        if not deploy_skill(task.skill, self.skill_type):
            return ExecutionResult(
                task_id=task.id,
                skill_type=self.skill_type.value,
                success=False,
                duration_seconds=0,
                turns=0,
                errors=[f"Failed to deploy skill: {task.skill}"],
                output="",
                quality_score=0
            )
        
        # Execute the task
        start_time = time.time()
        result = execute_with_claude_code(
            prompt=task.prompt,
            timeout=task.timeout_seconds
        )
        duration = time.time() - start_time
        
        # Verify output
        success, quality_score = verify_output(result, task)
        
        # Extract additional info
        turns = result.get("turns", 1) if isinstance(result, dict) else 1
        errors = []
        if not success:
            errors = [result.get("error", "Unknown error")] if isinstance(result, dict) else ["Execution failed"]
        
        execution_result = ExecutionResult(
            task_id=task.id,
            skill_type=self.skill_type.value,
            success=success,
            duration_seconds=duration,
            turns=turns,
            errors=errors,
            output=str(result)[:500] if isinstance(result, dict) else str(result)[:500],
            quality_score=quality_score
        )
        
        if success:
            log_success(f"Task {task.id} completed successfully ({duration:.1f}s)")
        else:
            log_error(f"Task {task.id} failed: {errors}")
        
        return execution_result
    
    def run_experiment(self, tasks: List[Task], limit: Optional[int] = None) -> ExperimentReport:
        """Run the full experiment."""
        started_at = datetime.now().isoformat()
        
        # Shuffle tasks for randomization
        shuffled_tasks = random.sample(tasks, len(tasks))
        
        # Apply limit
        if limit:
            shuffled_tasks = shuffled_tasks[:limit]
        
        log_info(f"Starting experiment with {len(shuffled_tasks)} tasks ({self.skill_type.value})")
        
        # Clear existing skills
        clear_skills()
        
        # Run each task
        for i, task in enumerate(shuffled_tasks):
            log_info(f"[{i+1}/{len(shuffled_tasks)}] Processing {task.id}")
            
            result = self.run_task(task)
            self.results.append(result)
            
            # Save intermediate result
            self._save_result(result)
            
            # Brief pause between tasks
            time.sleep(2)
        
        completed_at = datetime.now().isoformat()
        
        # Generate report
        report = self._generate_report(started_at, completed_at)
        
        return report
    
    def _save_result(self, result: ExecutionResult) -> None:
        """Save a single result to file."""
        result_file = RESULTS_DIR / f"{self.experiment_id}_{result.task_id}.json"
        with open(result_file, "w", encoding="utf-8") as f:
            json.dump(asdict(result), f, indent=2)
    
    def _generate_report(self, started_at: str, completed_at: str) -> ExperimentReport:
        """Generate the experiment report."""
        total = len(self.results)
        success_count = sum(1 for r in self.results if r.success)
        
        report = ExperimentReport(
            experiment_id=self.experiment_id,
            skill_type=self.skill_type.value,
            total_tasks=total,
            success_count=success_count,
            success_rate=success_count / total * 100 if total > 0 else 0,
            avg_duration_seconds=sum(r.duration_seconds for r in self.results) / total if total > 0 else 0,
            avg_turns=sum(r.turns for r in self.results) / total if total > 0 else 0,
            avg_quality_score=sum(r.quality_score for r in self.results) / total if total > 0 else 0,
            results=[asdict(r) for r in self.results],
            started_at=started_at,
            completed_at=completed_at
        )
        
        # Save report
        report_file = REPORTS_DIR / f"{self.experiment_id}_report.json"
        with open(report_file, "w", encoding="utf-8") as f:
            json.dump(asdict(report), f, indent=2)
        
        log_info(f"Report saved to {report_file}")
        
        return report


def run_comparative_study(tasks: List[Task], limit: Optional[int] = None) -> Dict[str, Any]:
    """Run comparative study between original and compiled skills."""
    log_info("=" * 60)
    log_info("COMPARATIVE STUDY: Original vs Compiled Skills")
    log_info("=" * 60)
    
    # Phase 1: Original skills
    log_info("\nPhase 1: Testing with ORIGINAL skills")
    log_info("-" * 40)
    evaluator_original = AcademicEvaluator(SkillType.ORIGINAL)
    report_original = evaluator_original.run_experiment(tasks, limit)
    
    # Wait between phases
    log_info("\nWaiting 30 seconds before next phase...")
    time.sleep(30)
    
    # Phase 2: Compiled skills
    log_info("\nPhase 2: Testing with COMPILED skills")
    log_info("-" * 40)
    evaluator_compiled = AcademicEvaluator(SkillType.COMPILED)
    report_compiled = evaluator_compiled.run_experiment(tasks, limit)
    
    # Generate comparison
    comparison = {
        "experiment_id": f"COMP-{datetime.now().strftime('%Y%m%d-%H%M%S')}",
        "original": asdict(report_original),
        "compiled": asdict(report_compiled),
        "comparison": {
            "success_rate_diff": report_compiled.success_rate - report_original.success_rate,
            "success_rate_diff_pct": (report_compiled.success_rate - report_original.success_rate) / report_original.success_rate * 100 if report_original.success_rate > 0 else 0,
            "duration_diff": report_original.avg_duration_seconds - report_compiled.avg_duration_seconds,
            "duration_diff_pct": (report_original.avg_duration_seconds - report_compiled.avg_duration_seconds) / report_original.avg_duration_seconds * 100 if report_original.avg_duration_seconds > 0 else 0,
            "quality_diff": report_compiled.avg_quality_score - report_original.avg_quality_score,
        },
        "conclusion": _generate_conclusion(report_original, report_compiled)
    }
    
    # Save comparison report
    comparison_file = REPORTS_DIR / f"comparison_{comparison['experiment_id']}.json"
    with open(comparison_file, "w", encoding="utf-8") as f:
        json.dump(comparison, f, indent=2)
    
    log_info(f"\nComparison report saved to {comparison_file}")
    
    return comparison


def _generate_conclusion(original: ExperimentReport, compiled: ExperimentReport) -> str:
    """Generate a conclusion based on the comparison."""
    conclusions = []
    
    if compiled.success_rate > original.success_rate:
        diff = compiled.success_rate - original.success_rate
        conclusions.append(f"Compiled skills show {diff:.1f}% higher success rate")
    elif compiled.success_rate < original.success_rate:
        diff = original.success_rate - compiled.success_rate
        conclusions.append(f"Original skills show {diff:.1f}% higher success rate")
    else:
        conclusions.append("Success rates are equivalent")
    
    if compiled.avg_duration_seconds < original.avg_duration_seconds:
        diff = (original.avg_duration_seconds - compiled.avg_duration_seconds) / original.avg_duration_seconds * 100
        conclusions.append(f"Compiled skills are {diff:.1f}% faster")
    elif compiled.avg_duration_seconds > original.avg_duration_seconds:
        diff = (compiled.avg_duration_seconds - original.avg_duration_seconds) / original.avg_duration_seconds * 100
        conclusions.append(f"Original skills are {diff:.1f}% faster")
    
    if compiled.avg_quality_score > original.avg_quality_score:
        conclusions.append(f"Compiled skills produce higher quality output ({compiled.avg_quality_score:.2f} vs {original.avg_quality_score:.2f})")
    
    return "; ".join(conclusions) if conclusions else "No significant difference detected"


def print_summary(report: ExperimentReport) -> None:
    """Print a summary of the experiment."""
    print("\n" + "=" * 60)
    print(f"EXPERIMENT SUMMARY: {report.skill_type.upper()}")
    print("=" * 60)
    print(f"Experiment ID: {report.experiment_id}")
    print(f"Total Tasks: {report.total_tasks}")
    print(f"Success Count: {report.success_count}")
    print(f"Success Rate: {report.success_rate:.1f}%")
    print(f"Avg Duration: {report.avg_duration_seconds:.1f}s")
    print(f"Avg Quality Score: {report.avg_quality_score:.2f}/5")
    print("=" * 60)


# ============================================================================
# Main Entry Point
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Academic Evaluation Runner for Nexa Skill Compiler"
    )
    parser.add_argument(
        "--mode",
        choices=["original", "compiled", "compare"],
        default="compare",
        help="Evaluation mode: original, compiled, or compare (default: compare)"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit number of tasks to run"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all tasks (no limit)"
    )
    parser.add_argument(
        "--category",
        choices=["document", "code", "data", "swe_bench"],
        default=None,
        help="Run only tasks of specific category"
    )
    
    args = parser.parse_args()
    
    # Ensure directories exist
    ensure_directories()
    
    # Get task definitions
    all_tasks = get_task_definitions()
    
    # Filter by category if specified
    if args.category:
        all_tasks = [t for t in all_tasks if t.category == args.category]
    
    # Set limit
    limit = None if args.all else args.limit
    
    log_info(f"Total tasks available: {len(all_tasks)}")
    
    if args.mode == "compare":
        comparison = run_comparative_study(all_tasks, limit)
        
        print("\n" + "=" * 60)
        print("COMPARATIVE STUDY RESULTS")
        print("=" * 60)
        print(f"Original Success Rate: {comparison['original']['success_rate']:.1f}%")
        print(f"Compiled Success Rate: {comparison['compiled']['success_rate']:.1f}%")
        print(f"Success Rate Difference: {comparison['comparison']['success_rate_diff']:.1f}%")
        print(f"Duration Difference: {comparison['comparison']['duration_diff_pct']:.1f}%")
        print(f"Quality Difference: {comparison['comparison']['quality_diff']:.2f}")
        print(f"\nConclusion: {comparison['conclusion']}")
        print("=" * 60)
        
    else:
        skill_type = SkillType.ORIGINAL if args.mode == "original" else SkillType.COMPILED
        evaluator = AcademicEvaluator(skill_type)
        report = evaluator.run_experiment(all_tasks, limit)
        print_summary(report)


if __name__ == "__main__":
    main()