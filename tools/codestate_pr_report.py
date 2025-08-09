#!/usr/bin/env python3
"""
Generate a PR report for changed files: language distribution, LOC stats, and a simple complexity heatmap.
This script is standalone (stdlib only) to avoid runtime deps. It scans only the changed files between two git SHAs.

Outputs:
- codestate_pr_report.md : Markdown content suitable for PR comments
- codestate_pr_report.html : An HTML artifact with a detailed table

Env (recommended):
- BASE_SHA: base commit SHA (e.g., github.event.pull_request.base.sha)
- HEAD_SHA: head commit SHA (e.g., github.sha)
"""
from __future__ import annotations

import os
import re
import sys
import json
import html
import shlex
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple


SUPPORTED_EXTS = {
    '.py', '.js', '.ts', '.jsx', '.tsx', '.mjs', '.cjs',
    '.java', '.c', '.h', '.cpp', '.hpp', '.cc', '.cs', '.go', '.rb', '.php', '.rs', '.kt', '.swift',
    '.m', '.mm', '.scala', '.sh', '.bash', '.zsh', '.ps1', '.psm1', '.pl', '.pm', '.r', '.jl', '.lua',
    '.ex', '.exs', '.hs', '.erl', '.clj', '.groovy', '.dart', '.sql', '.css', '.scss', '.sass', '.less',
    '.html', '.vue', '.svelte', '.hbs', '.ejs', '.jinja', '.jinja2', '.njk'
}


def run(cmd: str) -> Tuple[int, str, str]:
    p = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = p.communicate()
    return p.returncode, out, err


def list_changed_files(base: str, head: str) -> List[str]:
    # Include Added, Copied, Modified, Renamed, Type changed, Unmerged, Unknown, Broken
    cmd = f"git diff --name-only --diff-filter=ACMRTUXB {shlex.quote(base)}..{shlex.quote(head)}"
    code, out, err = run(cmd)
    if code != 0:
        print(f"::warning::Failed to list changed files: {err.strip()}\nFalling back to 'git diff --name-only' on HEAD.")
        code, out, _ = run("git diff --name-only")
    files = [line.strip() for line in out.splitlines() if line.strip()]
    # Filter only supported extensions and existing files
    result = []
    for f in files:
        p = Path(f)
        if p.suffix.lower() in SUPPORTED_EXTS and p.exists() and p.is_file():
            result.append(str(p))
    return result


def estimate_comment_lines(ext: str, lines: List[str]) -> int:
    ext = ext.lower()
    comment = 0
    in_block = False
    block_start = {'/*', "'''", '"""'}
    block_end = {'*/', "'''", '"""'}
    for raw in lines:
        s = raw.strip()
        if not s:
            continue
        if in_block:
            comment += 1
            # Heuristic: close on end token appearance
            if any(tok in s for tok in block_end):
                in_block = False
            continue
        # Single-line styles
        if ext in {'.py'}:
            if s.startswith('#'):
                comment += 1
                continue
            # Start of docstring block
            if s.startswith("'''") or s.startswith('"""'):
                comment += 1
                if not (s.endswith("'''") or s.endswith('"""')):
                    in_block = True
                continue
        else:
            if s.startswith('//'):
                comment += 1
                continue
            if s.startswith('/*') or s.startswith('<!--'):
                comment += 1
                in_block = True
                continue
    return comment


FUNC_PATTERNS = [
    re.compile(r"\bdef\s+\w+\s*\(", re.I),
    re.compile(r"\bfunction\b|=>\s*\(", re.I),
    re.compile(r"\b(class|interface)\s+\w+\b", re.I),
]

COMPLEXITY_TOKENS = (
    ' if ', ' for ', ' while ', ' case ', ' when ', ' elif ', ' switch ', ' catch ', ' try ',
    '&&', '||', '?:', '?', ' await ', ' async ', ' yield ', ' except ', ' with ', ' and ', ' or '
)


def count_functions(text: str) -> int:
    total = 0
    for pat in FUNC_PATTERNS:
        total += len(pat.findall(text))
    return total


def estimate_complexity(lines: List[str]) -> float:
    score = 0
    for raw in lines:
        s = f" {raw.strip()} ".lower()
        score += sum(1 for tok in COMPLEXITY_TOKENS if tok in s)
    return float(score)


def read_lines(path: Path) -> List[str]:
    try:
        return path.read_text(encoding='utf-8', errors='ignore').splitlines()
    except Exception:
        try:
            return path.read_text(encoding='latin-1', errors='ignore').splitlines()
        except Exception:
            return []


def analyze_files(files: List[str]) -> Tuple[Dict[str, dict], List[dict]]:
    by_ext: Dict[str, dict] = {}
    per_file: List[dict] = []
    for f in files:
        p = Path(f)
        ext = p.suffix.lower() or '<none>'
        lines = read_lines(p)
        total_lines = len(lines)
        comments = estimate_comment_lines(ext, lines)
        text = "\n".join(lines)
        func_count = count_functions(text)
        complexity = estimate_complexity(lines)
        pf = {
            'file': f,
            'ext': ext,
            'total_lines': total_lines,
            'comment_lines': comments,
            'function_count': func_count,
            'complexity': complexity,
        }
        per_file.append(pf)
        agg = by_ext.setdefault(ext, {
            'ext': ext, 'file_count': 0, 'total_lines': 0,
            'comment_lines': 0, 'function_count': 0
        })
        agg['file_count'] += 1
        agg['total_lines'] += total_lines
        agg['comment_lines'] += comments
        agg['function_count'] += func_count
    return by_ext, per_file


def make_bar(value: int, max_value: int, width: int = 24) -> str:
    if max_value <= 0:
        return ''
    n = int(round((value / max_value) * width))
    return '█' * max(1, n) if value > 0 else ''


def render_markdown(by_ext: Dict[str, dict], per_file: List[dict]) -> str:
    title = "## CodeState PR Report — Changed Files"
    if not per_file:
        return f"{title}\n\nNo supported code changes detected."

    # Extension summary table
    exts = sorted(by_ext.values(), key=lambda x: (-x['total_lines'], x['ext']))
    header = "| ext | files | lines | comments | functions |\n|---|---:|---:|---:|---:|"
    rows = [
        f"| {e['ext']} | {e['file_count']} | {e['total_lines']} | {e['comment_lines']} | {e['function_count']} |"
        for e in exts
    ]

    # Language distribution bars (by lines)
    max_lines = max((e['total_lines'] for e in exts), default=0)
    bars = [f"`{e['ext']}` {make_bar(e['total_lines'], max_lines)} {e['total_lines']}" for e in exts]

    # Complexity heatmap (top 10)
    top_complex = sorted(per_file, key=lambda x: (-x['complexity'], -x['total_lines']))[:10]
    max_c = int(max((f['complexity'] for f in top_complex), default=0))
    heat = [
        f"`{Path(f['file']).name}` {make_bar(int(f['complexity']), max_c)} {int(f['complexity'])}"
        for f in top_complex
    ]

    md = []
    md.append(title)
    md.append("")
    md.append("### Summary by extension")
    md.append(header)
    md.extend(rows)
    md.append("")
    md.append("### Language distribution (by lines)")
    md.extend([f"- {b}" for b in bars])
    md.append("")
    md.append("### Complexity heatmap (top 10 files)")
    if heat:
        md.extend([f"- {h}" for h in heat])
    else:
        md.append("- No files to display")
    md.append("")
    md.append(
        "> Generated by CodeState PR reporter. An HTML report has been uploaded as a workflow artifact."
    )
    return "\n".join(md)


def render_html(by_ext: Dict[str, dict], per_file: List[dict]) -> str:
    html_rows = []
    for f in sorted(per_file, key=lambda x: (-x['total_lines'], x['file'])):
        html_rows.append(
            f"<tr><td>{html.escape(f['file'])}</td><td>{html.escape(f['ext'])}</td>"
            f"<td style='text-align:right'>{f['total_lines']}</td>"
            f"<td style='text-align:right'>{f['comment_lines']}</td>"
            f"<td style='text-align:right'>{f['function_count']}</td>"
            f"<td style='text-align:right'>{int(f['complexity'])}</td></tr>"
        )
    ext_rows = []
    for e in sorted(by_ext.values(), key=lambda x: (-x['total_lines'], x['ext'])):
        ext_rows.append(
            f"<tr><td>{html.escape(e['ext'])}</td>"
            f"<td style='text-align:right'>{e['file_count']}</td>"
            f"<td style='text-align:right'>{e['total_lines']}</td>"
            f"<td style='text-align:right'>{e['comment_lines']}</td>"
            f"<td style='text-align:right'>{e['function_count']}</td></tr>"
        )
    return f"""
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>CodeState PR Report</title>
  <style>
    body {{ font-family: -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; padding: 16px; }}
    table {{ border-collapse: collapse; width: 100%; margin-bottom: 24px; }}
    th, td {{ border: 1px solid #e5e7eb; padding: 8px; }}
    th {{ background: #f3f4f6; text-align: left; }}
    caption {{ text-align: left; font-weight: 600; margin: 8px 0; }}
    .muted {{ color: #6b7280; }}
  </style>
  </head>
<body>
  <h2>CodeState PR Report — Changed Files</h2>
  <p class="muted">This artifact lists per-file and per-extension metrics for the current pull request.</p>
  <table>
    <caption>Summary by extension</caption>
    <thead>
      <tr><th>ext</th><th>files</th><th>lines</th><th>comments</th><th>functions</th></tr>
    </thead>
    <tbody>
      {''.join(ext_rows)}
    </tbody>
  </table>
  <table>
    <caption>Per-file details</caption>
    <thead>
      <tr><th>file</th><th>ext</th><th>lines</th><th>comments</th><th>functions</th><th>complexity</th></tr>
    </thead>
    <tbody>
      {''.join(html_rows)}
    </tbody>
  </table>
  <p class="muted">Generated by CodeState PR reporter.</p>
</body>
</html>
"""


def main() -> int:
    base = os.getenv('BASE_SHA') or os.getenv('GITHUB_BASE_SHA') or ''
    head = os.getenv('HEAD_SHA') or os.getenv('GITHUB_HEAD_SHA') or ''
    if not base or not head:
        # Try to infer base from origin or default branch
        # Fallback to HEAD~1..HEAD
        code, out, _ = run("git rev-parse HEAD")
        head = out.strip() if out.strip() else head
        code, out, _ = run("git rev-parse HEAD~1")
        base = out.strip() if out.strip() else base

    files = list_changed_files(base, head)
    by_ext, per_file = analyze_files(files)

    md = render_markdown(by_ext, per_file)
    html_content = render_html(by_ext, per_file)

    Path('codestate_pr_report.md').write_text(md, encoding='utf-8')
    Path('codestate_pr_report.html').write_text(html_content, encoding='utf-8')

    # Optional: also write a compact JSON for future automation
    payload = {
        'base': base, 'head': head, 'file_count': len(per_file),
        'by_ext': list(by_ext.values()), 'per_file': per_file,
    }
    Path('codestate_pr_report.json').write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding='utf-8')

    print("Report files generated: codestate_pr_report.md, codestate_pr_report.html, codestate_pr_report.json")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
