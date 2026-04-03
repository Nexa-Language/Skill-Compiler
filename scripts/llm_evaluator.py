#!/usr/bin/env python3
"""
Nexa Skill Compiler - LLM Evaluator

This script uses LLM to evaluate the quality of compiled skill outputs,
measuring semantic preservation and cross-platform consistency.
"""

import json
import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import requests
from dotenv import load_dotenv


@dataclass
class EvaluationResult:
    """LLM evaluation result"""
    skill_name: str
    platform: str
    semantic_score: float  # 0-100
    completeness_score: float  # 0-100
    consistency_score: float  # 0-100
    feedback: str
    duration_ms: float


class LLMEvaluator:
    """LLM-based evaluator for skill compilation quality"""

    def __init__(self, api_key: str, api_base: str, model: str = "gpt-4o-mini"):
        self.api_key = api_key
        self.api_base = api_base.rstrip("/")
        self.model = model
        self.results: list[EvaluationResult] = []

    def _call_llm(self, prompt: str) -> tuple[str, float]:
        """Call LLM API and return response with duration"""
        url = f"{self.api_base}/chat/completions"
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        data = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are an expert at evaluating AI agent skills. Provide scores as JSON."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        }

        start_time = time.time()
        response = requests.post(url, headers=headers, json=data, timeout=60)
        duration_ms = (time.time() - start_time) * 1000

        response.raise_for_status()
        result = response.json()
        return result["choices"][0]["message"]["content"], duration_ms

    def evaluate_semantic_preservation(
        self,
        original_skill: str,
        compiled_output: str,
        platform: str
    ) -> EvaluationResult:
        """Evaluate how well the compiled output preserves the original skill's semantics"""

        prompt = f"""Evaluate the following compiled skill output for semantic preservation.

ORIGINAL SKILL (SKILL.md):
```
{original_skill[:3000]}
```

COMPILED OUTPUT ({platform}):
```
{compiled_output[:3000]}
```

Rate the following aspects on a scale of 0-100:
1. **Semantic Score**: How well does the compiled output preserve the original skill's intent and meaning?
2. **Completeness Score**: How complete is the compiled output? Are all important sections preserved?
3. **Consistency Score**: Is the output internally consistent and well-structured?

Respond in JSON format:
{{
    "semantic_score": <0-100>,
    "completeness_score": <0-100>,
    "consistency_score": <0-100>,
    "feedback": "<Brief explanation of scores>"
}}"""

        try:
            response, duration = self._call_llm(prompt)
            # Parse JSON from response
            # Handle potential markdown code blocks
            if "```json" in response:
                response = response.split("```json")[1].split("```")[0]
            elif "```" in response:
                response = response.split("```")[1].split("```")[0]

            scores = json.loads(response.strip())

            return EvaluationResult(
                skill_name="unknown",
                platform=platform,
                semantic_score=float(scores.get("semantic_score", 0)),
                completeness_score=float(scores.get("completeness_score", 0)),
                consistency_score=float(scores.get("consistency_score", 0)),
                feedback=scores.get("feedback", ""),
                duration_ms=duration
            )
        except Exception as e:
            return EvaluationResult(
                skill_name="unknown",
                platform=platform,
                semantic_score=0,
                completeness_score=0,
                consistency_score=0,
                feedback=f"Evaluation failed: {str(e)}",
                duration_ms=0
            )

    def evaluate_cross_platform_consistency(
        self,
        claude_output: str,
        codex_output: str,
        gemini_output: str
    ) -> dict:
        """Evaluate consistency across different platform outputs"""

        prompt = f"""Evaluate the consistency of these compiled outputs for the same skill across different platforms.

CLAUDE OUTPUT:
```
{claude_output[:2000]}
```

CODEX OUTPUT:
```
{codex_output[:2000]}
```

GEMINI OUTPUT:
```
{gemini_output[:2000]}
```

Rate the cross-platform consistency on a scale of 0-100.
Consider: Do all platforms convey the same intent? Are procedures equivalent?

Respond in JSON format:
{{
    "consistency_score": <0-100>,
    "platforms_aligned": true/false,
    "differences": ["<list of key differences>"],
    "feedback": "<Brief explanation>"
}}"""

        try:
            response, _ = self._call_llm(prompt)
            if "```json" in response:
                response = response.split("```json")[1].split("```")[0]
            elif "```" in response:
                response = response.split("```")[1].split("```")[0]

            return json.loads(response.strip())
        except Exception as e:
            return {
                "consistency_score": 0,
                "platforms_aligned": False,
                "differences": [],
                "feedback": f"Evaluation failed: {str(e)}"
            }


def run_evaluation(project_root: Path) -> bool:
    """Run LLM evaluation on compiled outputs"""

    # Load environment variables
    load_dotenv(project_root / ".env")

    api_key = os.getenv("OPENAI_API_KEY")
    api_base = os.getenv("OPENAI_API_BASE", "https://api.openai.com/v1")

    if not api_key:
        print("❌ OPENAI_API_KEY not found in environment")
        return False

    print("🤖 LLM Evaluator for Nexa Skill Compiler\n")
    print(f"API Base: {api_base}")
    print(f"Model: gpt-4o-mini\n")

    evaluator = LLMEvaluator(api_key, api_base)

    # Find compiled outputs
    output_dir = project_root / "target" / "e2e-output"

    if not output_dir.exists():
        print("❌ No compiled outputs found. Run E2E tests first.")
        return False

    results = []

    # Evaluate each skill
    for skill_dir in output_dir.iterdir():
        if not skill_dir.is_dir():
            continue

        skill_name = skill_dir.name
        print(f"📊 Evaluating: {skill_name}")

        # Load original skill
        original_path = project_root / "tests" / "fixtures" / f"{skill_name}.md"
        if not original_path.exists():
            print(f"   ⚠️  Original skill not found, skipping")
            continue

        original_content = original_path.read_text()

        # Evaluate each platform output
        for platform in ["claude", "codex", "gemini"]:
            platform_file = skill_dir / platform / f"{skill_name}.xml" if platform == "claude" else \
                           skill_dir / platform / f"{skill_name}_schema.json" if platform == "codex" else \
                           skill_dir / platform / f"{skill_name}.md"

            if not platform_file.exists():
                # Try alternative paths
                platform_file = skill_dir / f"{skill_name}.{platform}.xml" if platform == "claude" else \
                               skill_dir / f"{skill_name}.{platform}.json" if platform == "codex" else \
                               skill_dir / f"{skill_name}.{platform}.md"

            if platform_file.exists():
                compiled_content = platform_file.read_text()
                result = evaluator.evaluate_semantic_preservation(
                    original_content,
                    compiled_content,
                    platform
                )
                result.skill_name = skill_name
                results.append(result)

                print(f"   {platform}: semantic={result.semantic_score:.0f}, "
                      f"completeness={result.completeness_score:.0f}, "
                      f"consistency={result.consistency_score:.0f}")
            else:
                print(f"   {platform}: ⚠️  Output not found")

        print()

    # Print summary
    if results:
        print("=" * 60)
        print("\n📈 Evaluation Summary\n")

        avg_semantic = sum(r.semantic_score for r in results) / len(results)
        avg_completeness = sum(r.completeness_score for r in results) / len(results)
        avg_consistency = sum(r.consistency_score for r in results) / len(results)

        print(f"  Average Semantic Score: {avg_semantic:.1f}/100")
        print(f"  Average Completeness Score: {avg_completeness:.1f}/100")
        print(f"  Average Consistency Score: {avg_consistency:.1f}/100")

        # Save results
        results_file = output_dir / "llm-evaluation-results.json"
        results_data = {
            "summary": {
                "average_semantic_score": avg_semantic,
                "average_completeness_score": avg_completeness,
                "average_consistency_score": avg_consistency,
                "total_evaluations": len(results)
            },
            "evaluations": [
                {
                    "skill_name": r.skill_name,
                    "platform": r.platform,
                    "semantic_score": r.semantic_score,
                    "completeness_score": r.completeness_score,
                    "consistency_score": r.consistency_score,
                    "feedback": r.feedback,
                    "duration_ms": r.duration_ms
                }
                for r in results
            ]
        }

        with open(results_file, "w") as f:
            json.dump(results_data, f, indent=2)

        print(f"\n📄 Results saved to: {results_file}")

        # Quality gate
        if avg_semantic >= 70 and avg_completeness >= 80:
            print("\n✅ Quality gate PASSED")
            return True
        else:
            print("\n❌ Quality gate FAILED")
            return False

    return False


def main():
    """Main entry point"""
    project_root = Path(__file__).parent.parent

    try:
        success = run_evaluation(project_root)
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"❌ Evaluation failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()