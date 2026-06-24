#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from pathlib import Path


ROOT = Path(os.environ.get("ROOT", Path(__file__).resolve().parent))
OUT = Path(os.environ.get("OUT_DIR", ROOT / "out"))


def count_jsonl(path: Path) -> int:
    if not path.exists():
        return 0
    return sum(1 for line in path.read_text(errors="replace").splitlines() if line.strip())


def read_json(path: Path) -> dict:
    if not path.exists():
        return {"status": "missing"}
    try:
        return json.loads(path.read_text(errors="replace") or "{}")
    except json.JSONDecodeError as exc:
        return {"status": "invalid_json", "error": str(exc)}


def status_from_json(path: Path) -> str:
    data = read_json(path)
    if data.get("status"):
        return str(data["status"])
    if data.get("error"):
        return "error"
    if data.get("data") or data.get("videos"):
        return "ok"
    return "response"


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    errors = count_jsonl(OUT / "errors.jsonl")
    report = {
        "yt_dlp_metadata": {
            "records": count_jsonl(OUT / "yt_dlp_metadata.jsonl"),
            "status": "ok" if count_jsonl(OUT / "yt_dlp_metadata.jsonl") else "no_records",
        },
        "yt_dlp_download": {
            "records": count_jsonl(OUT / "yt_dlp_download.jsonl"),
        },
        "oembed": {
            "records": count_jsonl(OUT / "oembed.jsonl"),
            "status": "ok" if count_jsonl(OUT / "oembed.jsonl") else "no_records",
        },
        "display_api": {
            "status": status_from_json(OUT / "display_api_response.json"),
        },
        "research_api": {
            "status": status_from_json(OUT / "research_api_response.json"),
        },
        "data_portability_api": {
            "records": count_jsonl(OUT / "data_portability.jsonl"),
            "status": "not_probed",
        },
        "errors": {
            "records": errors,
        },
    }

    (OUT / "mechanism_report.json").write_text(json.dumps(report, indent=2) + "\n")

    lines = ["# TikTok Mechanism Probe Report", ""]
    for name, data in report.items():
        lines.append(f"## {name}")
        for key, value in data.items():
            lines.append(f"- {key}: `{value}`")
        lines.append("")
    (OUT / "mechanism_report.md").write_text("\n".join(lines))


if __name__ == "__main__":
    main()
