#!/usr/bin/env python3
"""Validate the docs/skills planning boundary; not a product test runner."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path
from urllib.parse import unquote, urlsplit

SKILLS = {
    "evobase-engineering",
    "evobase-code-review",
    "evobase-ui-engineering",
    "evobase-cmp-ui-engineering",
    "evobase-ui-inspector",
}


def allowed_path(path: str) -> bool:
    return path == "AGENTS.md" or path.startswith(
        (".agents/skills/evobase-", "docs/work-plan/")
    )


def heading_anchors(text: str) -> set[str]:
    anchors: set[str] = set()
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", text, re.MULTILINE):
        heading = re.sub(r"[`*_]", "", heading).lower()
        heading = re.sub(r"[^\w\s-]", "", heading)
        anchors.add(heading.replace(" ", "-"))
    return anchors


def validate(root: Path, changed: list[str] | None = None) -> dict[str, object]:
    root = root.resolve()
    errors: list[str] = []
    for name in sorted(SKILLS):
        path = root / ".agents" / "skills" / name / "SKILL.md"
        if not path.is_file():
            errors.append(f"missing skill: {name}")
            continue
        text = path.read_text(encoding="utf-8")
        frontmatter = re.match(r"\A---\n(.*?)\n---(?:\n|$)", text, re.DOTALL)
        if not frontmatter:
            errors.append(f"missing frontmatter: {name}")
        elif not re.search(rf"^name:\s*{re.escape(name)}\s*$", frontmatter[1], re.MULTILINE):
            errors.append(f"wrong frontmatter name: {name}")
        elif not re.search(r"^description:\s*\S", frontmatter[1], re.MULTILINE):
            errors.append(f"missing description: {name}")

    markdown = sorted((root / ".agents" / "skills").glob("evobase-*/**/*.md"))
    markdown += sorted((root / "docs" / "work-plan").rglob("*.md"))
    if (root / "AGENTS.md").is_file():
        markdown.append(root / "AGENTS.md")
    link_count = 0
    for path in markdown:
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root).as_posix()
        if not text.endswith("\n"):
            errors.append(f"missing final newline: {relative}")
        if any(line.rstrip() != line for line in text.splitlines()):
            errors.append(f"trailing whitespace: {relative}")
        if re.search(r"/(?:Users|home|root)/[^\s`]+", text):
            errors.append(f"personal absolute path: {relative}")
        if re.search(r"\b(?:\d{1,3}\.){3}\d{1,3}\b", text):
            errors.append(f"literal host IP: {relative}")
        if re.search(r"cargo\s+xtask\s+(?:deploy|verify|ui-inspect)\s+", text):
            errors.append(f"inherited runnable VOT command: {relative}")
        for destination in re.findall(r"(?<!!)\[[^\]]+\]\(([^\s)]+)\)", text):
            parsed = urlsplit(destination)
            if parsed.scheme or parsed.netloc:
                continue
            link_count += 1
            target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not target.is_relative_to(root):
                errors.append(f"link outside repository: {relative} -> {destination}")
            elif not target.exists():
                errors.append(f"missing link: {relative} -> {destination}")
            elif parsed.fragment and target.suffix == ".md":
                if unquote(parsed.fragment) not in heading_anchors(target.read_text(encoding="utf-8")):
                    errors.append(f"missing anchor: {relative} -> {destination}")

    if changed is None:
        commands = [
            ["git", "diff", "--name-only", "HEAD", "--"],
            ["git", "ls-files", "--others", "--exclude-standard"],
        ]
        changed = []
        for command in commands:
            result = subprocess.run(command, cwd=root, capture_output=True, text=True, check=False)
            if result.returncode:
                errors.append(f"scope inventory failed: {result.stderr.strip()}")
            changed.extend(result.stdout.splitlines())
    for path in sorted(set(changed)):
        if not allowed_path(path):
            errors.append(f"outside planning scope: {path}")

    return {
        "evidence": "statically-checked document metadata/links/privacy patterns/scope only",
        "markdown_files": len(markdown),
        "relative_links": link_count,
        "required_skills": len(SKILLS),
        "changed_paths": len(set(changed)),
        "errors": errors,
        "limitations": "No product compile, policy, DB, UI/native, provider or exhaustive secret scan proof",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    args = parser.parse_args()
    report = validate(args.root)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 1 if report["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
