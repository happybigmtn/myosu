#!/usr/bin/env python3
"""Evaluation harness for the myosu autoresearch loop.

Runs the experiment framework on a corpus subset, scores results,
and writes a JSON summary. This wraps `python3 main.py` and
aggregates the per-game primary_metric into a total score.

Inputs:
  --config    Path to solver architecture config JSON
  --corpus    Path to corpus JSON (smoke or full)
  --timeout   Per-game timeout in seconds (default: 300)
  --output    Path to write scored results JSON

Outputs (results JSON):
  {
    "summary": {
      "total_score": float,
      "successful_cases": int,
      "case_count": int,
      "timed_out_cases": int,
      "mean_primary_metric": float
    },
    "cases": [ { "game_id": str, "score": float, "success": bool, ... }, ... ]
  }
"""

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path


def run_single_game(config_path: Path, game: dict, timeout: int) -> dict:
    """Run the experiment framework for a single game and return scored result."""
    game_id = game["id"]
    game_name = game["name"]

    result = {
        "game_id": game_id,
        "game_name": game_name,
        "score": 0.0,
        "success": False,
        "timed_out": False,
        "error": None,
        "elapsed_seconds": 0.0,
    }

    start = time.monotonic()
    try:
        proc = subprocess.run(
            [
                sys.executable, "main.py",
                "--config", str(config_path),
                "--game", game_id,
            ],
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=config_path.parent,
        )
        result["elapsed_seconds"] = round(time.monotonic() - start, 2)

        if proc.returncode != 0:
            result["error"] = proc.stderr[-500:] if proc.stderr else f"rc={proc.returncode}"
            return result

        # main.py writes results to stdout as JSON with a primary_metric field
        output = json.loads(proc.stdout)
        result["score"] = float(output.get("primary_metric", 0.0))
        result["success"] = True

    except subprocess.TimeoutExpired:
        result["elapsed_seconds"] = round(time.monotonic() - start, 2)
        result["timed_out"] = True
        result["error"] = f"timed out after {timeout}s"
    except json.JSONDecodeError as exc:
        result["elapsed_seconds"] = round(time.monotonic() - start, 2)
        result["error"] = f"invalid JSON output: {exc}"
    except Exception as exc:
        result["elapsed_seconds"] = round(time.monotonic() - start, 2)
        result["error"] = str(exc)

    return result


def main() -> None:
    parser = argparse.ArgumentParser(description="Myosu autoresearch evaluation harness")
    parser.add_argument("--config", required=True, help="Path to solver config JSON")
    parser.add_argument("--corpus", required=True, help="Path to corpus JSON (smoke/full)")
    parser.add_argument("--timeout", type=int, default=300, help="Per-game timeout in seconds")
    parser.add_argument("--output", required=True, help="Path to write results JSON")
    args = parser.parse_args()

    config_path = Path(args.config).resolve()
    corpus_path = Path(args.corpus).resolve()
    output_path = Path(args.output).resolve()

    if not config_path.exists():
        sys.exit(f"config not found: {config_path}")
    if not corpus_path.exists():
        sys.exit(f"corpus not found: {corpus_path}")

    # Validate config is parseable JSON
    try:
        json.loads(config_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        sys.exit(f"invalid config JSON: {exc}")

    corpus = json.loads(corpus_path.read_text(encoding="utf-8"))
    games = corpus["games"]

    cases = []
    for game in games:
        result = run_single_game(config_path, game, args.timeout)
        cases.append(result)

    successful = [c for c in cases if c["success"]]
    timed_out = [c for c in cases if c["timed_out"]]
    total_score = sum(c["score"] for c in cases)
    mean_primary = total_score / len(cases) if cases else 0.0

    output = {
        "summary": {
            "total_score": round(total_score, 6),
            "successful_cases": len(successful),
            "case_count": len(cases),
            "timed_out_cases": len(timed_out),
            "mean_primary_metric": round(mean_primary, 6),
        },
        "cases": cases,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
