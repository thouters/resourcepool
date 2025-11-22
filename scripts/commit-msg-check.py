#!/usr/bin/env python3
"""Simple Conventional Commits check for pre-commit commit-msg hook.

This script expects the commit message file path as the first argument
and checks the first line against a Conventional Commits pattern.
"""
import re
import sys


def main(path: str) -> int:
    try:
        with open(path, "r", encoding="utf-8") as f:
            first = f.readline().rstrip("\n")
    except Exception as e:
        print(f"ERROR: Could not read commit message file: {e}")
        return 2

    # Conventional Commits: type(scope?): description
    pattern = re.compile(
        r"^(?:build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(?:\([^\)]+\))?:\s.+"
    )

    if pattern.match(first):
        return 0

    print("\nERROR: Commit message does not follow Conventional Commits format.\n")
    print("Expected format: <type>(optional-scope): <description>")
    print("Allowed types: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test")
    print(f"Your commit message first line: {first!r}\n")
    print("Examples:")
    print("  feat(parser): add support for X")
    print("  fix: correct off-by-one in widget")
    return 1


if __name__ == "__main__":
    if len(sys.argv) >= 2:
        sys.exit(main(sys.argv[1]))
    else:
        print("ERROR: commit-msg hook did not pass the commit message file path")
        sys.exit(2)
