#!/usr/bin/env python3
"""
Generate Task Input Data for Academic Evaluation

This script creates all necessary input files for the 50 evaluation tasks.

Usage:
    python scripts/generate_task_data.py
"""

import json
import os
import random
import string
from pathlib import Path
from datetime import datetime, timedelta


# ============================================================================
# Configuration
# ============================================================================

PROJECT_ROOT = Path(__file__).parent.parent
EXPERIMENTS_DIR = PROJECT_ROOT / "experiments"
TASKS_DIR = EXPERIMENTS_DIR / "tasks"
INPUT_DIR = TASKS_DIR / "input"
OUTPUT_DIR = TASKS_DIR / "output"


# ============================================================================
# Helper Functions
# ============================================================================

def ensure_directories():
    """Create necessary directories."""
    for d in [EXPERIMENTS_DIR, TASKS_DIR, INPUT_DIR, OUTPUT_DIR]:
        d.mkdir(parents=True, exist_ok=True)
    
    # Create subdirectories
    for subdir in ["txt", "json", "csv", "py", "mixed"]:
        (INPUT_DIR / subdir).mkdir(parents=True, exist_ok=True)


def random_string(length=10):
    """Generate random string."""
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))


def random_name():
    """Generate random name."""
    first_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack"]
    last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Wilson", "Taylor"]
    return f"{random.choice(first_names)} {random.choice(last_names)}"


def random_email():
    """Generate random email."""
    domains = ["gmail.com", "yahoo.com", "outlook.com", "example.com", "test.org"]
    return f"{random_string(8).lower()}@{random.choice(domains)}"


# ============================================================================
# Document Task Data Generators
# ============================================================================

def generate_word_sample():
    """Generate sample Word document content (as text representation)."""
    content = """# Sample Word Document

## Introduction
This is a sample document for testing the docx skill.

## Table Example
| Name | Age | City |
|------|-----|------|
"""
    for _ in range(5):
        content += f"| {random_name()} | {random.randint(20, 60)} | {random.choice(['New York', 'London', 'Tokyo', 'Paris', 'Berlin'])} |\n"
    
    content += """
## Conclusion
This document contains sample data for extraction testing.
"""
    return content


def generate_pdf_sample():
    """Generate sample PDF content (as text representation)."""
    content = f"""# PDF Document Sample

Generated: {datetime.now().strftime('%Y-%m-%d')}

## Abstract
This is a sample PDF document for testing text extraction capabilities.

## Content
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor 
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis 
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

## Data Section
- Item 1: {random_string(15)}
- Item 2: {random_string(15)}
- Item 3: {random_string(15)}

## References
1. Sample Reference A
2. Sample Reference B
3. Sample Reference C
"""
    return content


def generate_excel_sample():
    """Generate sample Excel data (as CSV representation)."""
    content = "ID,Name,Department,Salary,HireDate\n"
    departments = ["Engineering", "Marketing", "Sales", "HR", "Finance"]
    
    for i in range(1, 21):
        name = random_name()
        dept = random.choice(departments)
        salary = random.randint(50000, 150000)
        hire_date = datetime.now() - timedelta(days=random.randint(100, 3000))
        content += f"{i},{name},{dept},{salary},{hire_date.strftime('%Y-%m-%d')}\n"
    
    return content


def generate_pptx_sample():
    """Generate sample PowerPoint content."""
    content = """# Presentation Outline

## Slide 1: Title Slide
- Title: Sample Presentation
- Subtitle: Generated for Testing
- Author: Test User

## Slide 2: Introduction
- Point 1: Overview of the project
- Point 2: Key objectives
- Point 3: Timeline

## Slide 3: Main Content
- Detailed analysis
- Key findings
- Recommendations

## Slide 4: Conclusion
- Summary
- Next steps
- Q&A
"""
    return content


def generate_text_files():
    """Generate multiple text files for merging."""
    files = {}
    for i in range(1, 6):
        files[f"file_{i}.txt"] = f"""# File {i}

This is the content of file {i}.
Generated at: {datetime.now().isoformat()}

Content:
{random_string(100)}
"""
    return files


def generate_mixed_files():
    """Generate mixed file types for organization test."""
    files = {}
    
    # Text files
    for i in range(3):
        files[f"document_{i}.txt"] = f"Document {i} content: {random_string(50)}"
    
    # JSON files
    for i in range(2):
        files[f"data_{i}.json"] = json.dumps({
            "id": i,
            "name": random_name(),
            "value": random.randint(1, 100)
        }, indent=2)
    
    # CSV files
    for i in range(2):
        files[f"table_{i}.csv"] = "a,b,c\n1,2,3\n4,5,6\n"
    
    return files


# ============================================================================
# Code Task Data Generators
# ============================================================================

def generate_python_sample():
    """Generate sample Python code for analysis."""
    content = '''#!/usr/bin/env python3
"""Sample Python module for code analysis testing."""

import os
import sys
from typing import List, Dict, Optional

class DataProcessor:
    """Process data from various sources."""
    
    def __init__(self, name: str):
        self.name = name
        self.data = []
    
    def load_data(self, filepath: str) -> bool:
        """Load data from a file."""
        try:
            with open(filepath, 'r') as f:
                self.data = f.read().splitlines()
            return True
        except Exception as e:
            print(f"Error loading data: {e}")
            return False
    
    def process(self) -> Dict[str, int]:
        """Process the loaded data."""
        result = {
            "total": len(self.data),
            "unique": len(set(self.data)),
            "empty": sum(1 for d in self.data if not d.strip())
        }
        return result
    
    def filter_data(self, predicate) -> List[str]:
        """Filter data using a predicate function."""
        return [d for d in self.data if predicate(d)]


def calculate_statistics(numbers: List[float]) -> Dict[str, float]:
    """Calculate basic statistics for a list of numbers."""
    if not numbers:
        return {"mean": 0, "median": 0, "std": 0}
    
    n = len(numbers)
    mean = sum(numbers) / n
    sorted_nums = sorted(numbers)
    median = sorted_nums[n // 2] if n % 2 else (sorted_nums[n//2-1] + sorted_nums[n//2]) / 2
    variance = sum((x - mean) ** 2 for x in numbers) / n
    std = variance ** 0.5
    
    return {"mean": mean, "median": median, "std": std}


def main():
    processor = DataProcessor("test")
    processor.load_data("data.txt")
    stats = processor.process()
    print(f"Statistics: {stats}")


if __name__ == "__main__":
    main()
'''
    return content


def generate_buggy_python():
    """Generate Python code with bugs for fixing."""
    content = '''#!/usr/bin/env python3
"""Buggy Python code for testing bug fixing capabilities."""

def calculate_average(numbers):
    """Calculate average of numbers."""
    # Bug: No check for empty list
    total = 0
    for num in numbers:
        total += num
    return total / len(numbers)


def find_maximum(values):
    """Find maximum value."""
    # Bug: Incorrect indentation
    max_val = values[0]
    for v in values:
        if v > max_val:
        max_val = v  # Bug: Wrong indentation
    return max_val


def process_data(data):
    """Process data list."""
    result = []
    for i in range(len(data)):
        # Bug: Off-by-one error potential
        if data[i+1] > data[i]:
            result.append(data[i+1])
    return result


def merge_dicts(dict1, dict2):
    """Merge two dictionaries."""
    # Bug: Modifies original dict
    result = dict1
    result.update(dict2)
    return result


class Counter:
    """Simple counter class."""
    
    def __init__(self):
        self.count = 0
    
    def increment(self):
        # Bug: No return value
        self.count + 1
    
    def get_count(self):
        return self.count
'''
    return content


def generate_no_validation_python():
    """Generate Python code without input validation."""
    content = '''#!/usr/bin/env python3
"""Code without input validation."""

def divide_numbers(a, b):
    """Divide two numbers."""
    return a / b


def get_item(lst, index):
    """Get item from list."""
    return lst[index]


def process_user_input(data):
    """Process user input."""
    name = data["name"]
    age = data["age"]
    email = data["email"]
    return f"User: {name}, Age: {age}, Email: {email}"


def calculate_discount(price, discount_rate):
    """Calculate discounted price."""
    return price * (1 - discount_rate)


def read_file(filepath):
    """Read file contents."""
    with open(filepath, 'r') as f:
        return f.read()
'''
    return content


def generate_slow_python():
    """Generate slow Python code for optimization."""
    content = '''#!/usr/bin/env python3
"""Slow Python code for optimization testing."""

def find_duplicates_slow(items):
    """Find duplicates in a list - O(n^2) complexity."""
    duplicates = []
    for i in range(len(items)):
        for j in range(i + 1, len(items)):
            if items[i] == items[j] and items[i] not in duplicates:
                duplicates.append(items[i])
    return duplicates


def fibonacci_slow(n):
    """Calculate fibonacci - exponential complexity."""
    if n <= 1:
        return n
    return fibonacci_slow(n - 1) + fibonacci_slow(n - 2)


def sum_of_squares_slow(n):
    """Calculate sum of squares - inefficient."""
    total = 0
    for i in range(1, n + 1):
        total += i * i
    return total


def search_in_list_slow(lst, target):
    """Search in unsorted list - linear search."""
    for i in range(len(lst)):
        if lst[i] == target:
            return i
    return -1


def merge_lists_slow(list1, list2):
    """Merge two lists - inefficient."""
    result = list1[:]
    for item in list2:
        if item not in result:
            result.append(item)
    return result
'''
    return content


def generate_sync_python():
    """Generate synchronous Python code for async conversion."""
    content = '''#!/usr/bin/env python3
"""Synchronous code for async conversion testing."""

import time
import requests

def fetch_url(url):
    """Fetch URL synchronously."""
    response = requests.get(url)
    return response.text


def fetch_multiple_urls(urls):
    """Fetch multiple URLs one by one."""
    results = []
    for url in urls:
        result = fetch_url(url)
        results.append(result)
    return results


def process_data(data):
    """Process data synchronously."""
    time.sleep(1)  # Simulate processing
    return data.upper()


def save_to_file(data, filepath):
    """Save data to file synchronously."""
    with open(filepath, 'w') as f:
        f.write(data)


def main():
    urls = [
        "https://example.com/api/1",
        "https://example.com/api/2",
        "https://example.com/api/3",
    ]
    results = fetch_multiple_urls(urls)
    processed = [process_data(r) for r in results]
    save_to_file("\\n".join(processed), "output.txt")


if __name__ == "__main__":
    main()
'''
    return content


def generate_messy_imports_python():
    """Generate Python code with messy imports."""
    content = '''#!/usr/bin/env python3
"""Code with messy imports."""

import sys, os, json
from typing import *
import random
from collections import defaultdict, Counter
import requests
from pathlib import Path
import datetime
import time
from mymodule import something
import subprocess
from io import StringIO
import hashlib
import base64

def main():
    pass

if __name__ == "__main__":
    main()
'''
    return content


def generate_no_logging_python():
    """Generate Python code without logging."""
    content = '''#!/usr/bin/env python3
"""Code without logging."""

class DataProcessor:
    def __init__(self, config):
        self.config = config
        self.data = []
    
    def load(self, filepath):
        # No logging
        with open(filepath, 'r') as f:
            self.data = f.read().splitlines()
    
    def process(self):
        # No logging
        result = []
        for item in self.data:
            processed = item.strip().upper()
            result.append(processed)
        return result
    
    def save(self, filepath, data):
        # No logging
        with open(filepath, 'w') as f:
            f.write('\\n'.join(data))


def main():
    processor = DataProcessor({})
    processor.load("input.txt")
    result = processor.process()
    processor.save("output.txt", result)


if __name__ == "__main__":
    main()
'''
    return content


def generate_duplicate_code_python():
    """Generate Python code with duplicates for refactoring."""
    content = '''#!/usr/bin/env python3
"""Code with duplicate patterns."""

def process_user(user_data):
    """Process user data."""
    if not user_data:
        return None
    
    name = user_data.get("name", "")
    if not name:
        return None
    
    email = user_data.get("email", "")
    if not email:
        return None
    
    age = user_data.get("age", 0)
    if age < 0:
        return None
    
    return {
        "name": name.strip().title(),
        "email": email.strip().lower(),
        "age": age
    }


def process_customer(customer_data):
    """Process customer data."""
    if not customer_data:
        return None
    
    name = customer_data.get("name", "")
    if not name:
        return None
    
    email = customer_data.get("email", "")
    if not email:
        return None
    
    age = customer_data.get("age", 0)
    if age < 0:
        return None
    
    return {
        "name": name.strip().title(),
        "email": email.strip().lower(),
        "age": age
    }


def process_employee(employee_data):
    """Process employee data."""
    if not employee_data:
        return None
    
    name = employee_data.get("name", "")
    if not name:
        return None
    
    email = employee_data.get("email", "")
    if not email:
        return None
    
    age = employee_data.get("age", 0)
    if age < 0:
        return None
    
    return {
        "name": name.strip().title(),
        "email": email.strip().lower(),
        "age": age
    }
'''
    return content


# ============================================================================
# Data Task Generators
# ============================================================================

def generate_csv_data():
    """Generate sample CSV data."""
    content = "id,name,email,age,city,salary\n"
    cities = ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix"]
    
    for i in range(1, 101):
        name = random_name()
        email = random_email()
        age = random.randint(22, 65)
        city = random.choice(cities)
        salary = random.randint(40000, 200000)
        content += f"{i},{name},{email},{age},{city},{salary}\n"
    
    return content


def generate_json_data():
    """Generate sample JSON data."""
    data = {
        "config": {
            "app_name": "TestApp",
            "version": "1.0.0",
            "debug": True
        },
        "users": [
            {
                "id": i,
                "name": random_name(),
                "email": random_email(),
                "roles": random.sample(["admin", "user", "guest", "moderator"], k=random.randint(1, 3))
            }
            for i in range(1, 21)
        ],
        "settings": {
            "theme": random.choice(["light", "dark"]),
            "language": random.choice(["en", "zh", "es", "fr"]),
            "notifications": random.choice([True, False])
        }
    }
    return json.dumps(data, indent=2)


def generate_chart_data():
    """Generate data for chart visualization."""
    content = "category,value1,value2,value3\n"
    categories = ["A", "B", "C", "D", "E", "F", "G", "H"]
    
    for cat in categories:
        v1 = random.randint(10, 100)
        v2 = random.randint(10, 100)
        v3 = random.randint(10, 100)
        content += f"{cat},{v1},{v2},{v3}\n"
    
    return content


# ============================================================================
# Main Generation Function
# ============================================================================

def generate_all_data():
    """Generate all task input data."""
    print("Creating directories...")
    ensure_directories()
    
    # Document tasks
    print("Generating document task data...")
    (INPUT_DIR / "sample.docx.txt").write_text(generate_word_sample())
    (INPUT_DIR / "sample.pdf.txt").write_text(generate_pdf_sample())
    (INPUT_DIR / "data.xlsx.csv").write_text(generate_excel_sample())
    (INPUT_DIR / "presentation.pptx.txt").write_text(generate_pptx_sample())
    
    # Text files for merging
    print("Generating text files...")
    txt_dir = INPUT_DIR / "txt"
    for name, content in generate_text_files().items():
        (txt_dir / name).write_text(content)
    
    # Mixed files
    print("Generating mixed files...")
    mixed_dir = INPUT_DIR / "mixed"
    for name, content in generate_mixed_files().items():
        (mixed_dir / name).write_text(content)
    
    # Code tasks
    print("Generating code task data...")
    (INPUT_DIR / "sample.py").write_text(generate_python_sample())
    (INPUT_DIR / "buggy.py").write_text(generate_buggy_python())
    (INPUT_DIR / "no_validation.py").write_text(generate_no_validation_python())
    (INPUT_DIR / "slow.py").write_text(generate_slow_python())
    (INPUT_DIR / "sync.py").write_text(generate_sync_python())
    (INPUT_DIR / "messy_imports.py").write_text(generate_messy_imports_python())
    (INPUT_DIR / "no_logging.py").write_text(generate_no_logging_python())
    (INPUT_DIR / "duplicates.py").write_text(generate_duplicate_code_python())
    
    # Data tasks
    print("Generating data task data...")
    (INPUT_DIR / "data.csv").write_text(generate_csv_data())
    (INPUT_DIR / "config.json").write_text(generate_json_data())
    (INPUT_DIR / "chart_data.csv").write_text(generate_chart_data())
    
    # Create manifest
    manifest = {
        "generated_at": datetime.now().isoformat(),
        "files": {
            "document": ["sample.docx.txt", "sample.pdf.txt", "data.xlsx.csv", "presentation.pptx.txt"],
            "code": ["sample.py", "buggy.py", "no_validation.py", "slow.py", "sync.py", 
                     "messy_imports.py", "no_logging.py", "duplicates.py"],
            "data": ["data.csv", "config.json", "chart_data.csv"],
        },
        "total_files": sum(len(v) for v in [
            ["sample.docx.txt", "sample.pdf.txt", "data.xlsx.csv", "presentation.pptx.txt"],
            ["sample.py", "buggy.py", "no_validation.py", "slow.py", "sync.py", 
             "messy_imports.py", "no_logging.py", "duplicates.py"],
            ["data.csv", "config.json", "chart_data.csv"],
        ])
    }
    
    (INPUT_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2))
    
    print(f"\nGenerated {manifest['total_files']} input files in {INPUT_DIR}")
    print(f"Manifest saved to {INPUT_DIR / 'manifest.json'}")


if __name__ == "__main__":
    generate_all_data()