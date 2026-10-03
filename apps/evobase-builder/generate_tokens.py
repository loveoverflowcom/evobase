"""Canonical authored JSON -> deterministic Web adapter; no palette generation."""
from pathlib import Path
import argparse
import json

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "design/m3-expressive/source/tokens.json"


def generate():
    tokens = json.loads(SOURCE.read_text())
    lines = ["/* Generated from design/m3-expressive/source/tokens.json. Do not edit. */"]
    for theme, roles in tokens["color"].items():
        lines.append(":root {" if theme == "light" else ':root[data-theme="dark"] {')
        lines.extend(f"  --{name}: {value};" for name, value in roles.items())
        lines.append("}")
    lines.append(":root {")
    lines.append(f'  --font-family: "{tokens["type"]["fontFamily"]}", sans-serif;')
    for name, style in tokens["type"].items():
        if not isinstance(style, dict):
            continue
        for prop in ("size", "lineHeight", "weight"):
            value = style[prop]
            unit = "" if prop == "weight" else "px"
            lines.append(f"  --type-{name}-{prop}: {value}{unit};")
        if "numeric" in style:
            lines.append(f'  --type-{name}-numeric: {style["numeric"]};')
    for group in ("shape", "layout", "state", "motion"):
        for name, value in tokens[group].items():
            if isinstance(value, (int, float)):
                unit = "ms" if group == "motion" else "px"
                if group == "state" and name.endswith("Opacity"):
                    unit = ""
                lines.append(f"  --{group}-{name}: {value}{unit};")
            elif group == "motion" and name == "curve":
                lines.append(f"  --motion-curve: {value};")
    for value in tokens["space"]:
        lines.append(f"  --space-{value}: {value}px;")
    for name, value in tokens["elevation"].items():
        if isinstance(value, (int, float)):
            lines.append(f"  --elevation-{name}: {value};")
    lines.append("}")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = Path(__file__).parent / "tokens.css"
    generated = generate()
    if args.check:
        if output.read_text() != generated:
            raise SystemExit("Web token adapter drift: regenerate tokens.css")
        # Independent pinned roles catch accidental palette forks.
        assert "--primary: #3154B8;" in generated
        assert "--primary: #B5C5FF;" in generated
        assert "--type-data-size: 13px;" in generated
        assert "--layout-touchTarget: 48px;" in generated
        print("Canonical Web adapter: deterministic, pinned light/dark/data/target roles checked")
    else:
        output.write_text(generated)
