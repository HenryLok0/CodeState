"""
Visualizer module for ASCII chart output.
"""

import os
import csv
import io
import re

def ascii_bar_chart(data, value_key, label_key='ext', width=40, title=None):
    """
    Print an ASCII bar chart for the given data.
    data: list of dicts or tuples
    value_key: key for the value to visualize (e.g., 'total_lines')
    label_key: key for the label (e.g., file extension)
    width: max width of the bar
    title: optional chart title
    """
    if title:
        print(f"\n{title}")
    if not data:
        print("No data to display.")
        return
    max_value = max(item[value_key] for item in data)
    total = sum(item[value_key] for item in data)
    show_file_count = 'file_count' in data[0]
    for item in data:
        label = str(item[label_key]).ljust(8)
        value = item[value_key]
        bar_len = int((value / max_value) * width) if max_value else 0
        bar = '█' * bar_len
        percent = (value / total) * 100 if total else 0
        if show_file_count:
            print(f"{label} | {bar} {value} ({percent:.1f}%) [{item['file_count']} files]")
        else:
            print(f"{label} | {bar} {value} ({percent:.1f}%)")

def print_comment_density(data, label_key='ext'):
    """
    Print comment density as a percentage bar chart, skip 0% and show comment line count.
    """
    print("\nComment Density:")
    for item in data:
        label = str(item[label_key]).ljust(8)
        density = item.get('comment_density', 0)
        comment_lines = item.get('comment_lines', 0)
        percent = int(density * 100)
        if percent == 0:
            continue  # Skip 0%
        bar = '█' * (percent // 2)
        print(f"{label} | {bar} {percent}% ({comment_lines} lines)")

def ascii_pie_chart(data, value_key, label_key='ext', title=None):
    """
    Print an ASCII pie chart for language distribution.
    """
    if title:
        print(f"\n{title}")
    total = sum(item[value_key] for item in data)
    for item in data:
        label = str(item[label_key]).ljust(8)
        value = item[value_key]
        percent = (value / total) * 100 if total else 0
        pie = '●' * int(percent // 5)
        print(f"{label} | {pie} {percent:.1f}%")

def ascii_complexity_heatmap(file_details, title=None):
    """
    Print an ASCII heatmap for file/function complexity.
    file_details: list of per-file stats (from analyzer.get_file_details())
    """
    if title:
        print(f"\n{title}")
    if not file_details:
        print("No data to display.")
        return
    # Define thresholds (can be tuned)
    low = 1.5
    high = 3.0
    print(f"{'File':40} | {'Complexity':10} | Heatmap")
    print('-'*65)
    for f in file_details:
        cplx = f.get('complexity', 0)
        if cplx < low:
            symbol = '░'
        elif cplx < high:
            symbol = '▒'
        else:
            symbol = '▓'
        bar = symbol * min(int(cplx * 2), 40)
        print(f"{f['path'][:40]:40} | {cplx:10.2f} | {bar}")

def print_ascii_tree(root_path, max_depth=5, prefix=""):
    """
    Print an ASCII tree view of the directory structure.
    root_path: directory to print
    max_depth: maximum depth to display
    prefix: internal use for recursion
    """
    if max_depth < 0:
        return
    entries = []
    try:
        entries = sorted(os.listdir(root_path))
    except Exception:
        return
    entries = [e for e in entries if not e.startswith('.')]
    for idx, entry in enumerate(entries):
        path = os.path.join(root_path, entry)
        connector = "└── " if idx == len(entries) - 1 else "├── "
        print(prefix + connector + entry)
        if os.path.isdir(path):
            extension = "    " if idx == len(entries) - 1 else "│   "
            print_ascii_tree(path, max_depth-1, prefix + extension)

def html_report(data, title='Code Statistics'):
    """
    Export statistics as an HTML table.
    """
    html = [f'<h2>{title}</h2>', '<table border="1">']
    if data:
        headers = data[0].keys()
        html.append('<tr>' + ''.join(f'<th>{h}</th>' for h in headers) + '</tr>')
        for item in data:
            html.append('<tr>' + ''.join(f'<td>{item[h]}</td>' for h in headers) + '</tr>')
    html.append('</table>')
    return '\n'.join(html)

def markdown_report(data, title='Code Statistics'):
    """
    Export statistics as a Markdown table.
    """
    md = [f'## {title}\n']
    if data:
        headers = list(data[0].keys())
        md.append('|' + '|'.join(headers) + '|')
        md.append('|' + '|'.join(['---'] * len(headers)) + '|')
        for item in data:
            md.append('|' + '|'.join(str(item[h]) for h in headers) + '|')
    return '\n'.join(md)

def generate_markdown_summary(stats, health_report, hotspots=None):
    """
    Generate a markdown project summary from stats, health report, and hotspots.
    """
    lines = []
    lines.append('# Project Code Summary')
    lines.append('')
    lines.append('## Overall Statistics')
    lines.append('| Extension | Files | Lines | Comments | Functions | TODOs |')
    lines.append('|-----------|-------|-------|----------|-----------|-------|')
    for ext, info in stats.items():
        lines.append(f"| {ext} | {info['file_count']} | {info['total_lines']} | {info['comment_lines']} | {info['function_count']} | {info.get('todo_count', 0)} |")
    lines.append('')
    if health_report:
        lines.append('## Project Health')
        lines.append(f"- **Health Score:** {health_report['score']} / 100")
        lines.append(f"- **Average Comment Density:** {health_report['avg_comment_density']:.2%}")
        lines.append(f"- **Average Function Complexity:** {health_report['avg_complexity']:.2f}")
        lines.append(f"- **TODO/FIXME Count:** {health_report['todo_count']}")
        lines.append(f"- **Naming Violations:** {health_report['naming_violations']}")
        lines.append(f"- **Duplicate Code Blocks:** {health_report['duplicate_blocks']}")
        lines.append(f"- **Large Files:** {health_report['large_files']}")
        lines.append(f"- **Large Functions:** {health_report['large_functions']}")
        if health_report['suggestions']:
            lines.append('### Suggestions:')
            for s in health_report['suggestions']:
                lines.append(f"- {s}")
    if hotspots:
        lines.append('')
        lines.append('## Git Hotspots (Most Frequently Changed Files)')
        lines.append('| File | Commits |')
        lines.append('|------|---------|')
        for path, count in hotspots:
            lines.append(f"| {path} | {count} |")
    return '\n'.join(lines)

def format_size(num_bytes):
    """
    Format file size in bytes to KB/MB/GB as appropriate.
    """
    if num_bytes >= 1024**3:
        return f"{num_bytes / (1024**3):.2f} GB"
    elif num_bytes >= 1024**2:
        return f"{num_bytes / (1024**2):.2f} MB"
    elif num_bytes >= 1024:
        return f"{num_bytes / 1024:.2f} KB"
    else:
        return f"{num_bytes} B"

def print_table(rows, headers=None, title=None):
    """
    Print a list of dicts as a pretty aligned table.
    """
    if not rows:
        print("No data to display.")
        return
    if headers is None:
        headers = list(rows[0].keys())
    # If showing contributor stats, calculate workload_score and percent, sort by workload_score
    if all(h in headers for h in ['commit_count', 'line_count', 'file_count']):
        try:
            for r in rows:
                r['_workload_score'] = (
                    0.5 * int(r.get('line_count', 0)) +
                    0.3 * int(r.get('commit_count', 0)) +
                    0.2 * int(r.get('file_count', 0))
                )
            total_score = sum(r['_workload_score'] for r in rows)
            for r in rows:
                if total_score > 0:
                    r['workload_percent'] = f"{(r['_workload_score'] / total_score * 100):.1f}%"
                else:
                    r['workload_percent'] = '0.0%'
            rows = sorted(rows, key=lambda r: r['_workload_score'], reverse=True)
            if 'workload_percent' not in headers:
                headers.append('workload_percent')
        except Exception:
            pass  # Fallback: do not sort or add percent if error
    # Format size column if present
    formatted_rows = []
    for row in rows:
        new_row = dict(row)
        if 'size' in new_row:
            try:
                new_row['size'] = format_size(int(new_row['size']))
            except Exception:
                pass
        formatted_rows.append(new_row)
    col_widths = [max(len(str(h)), max(len(str(row.get(h, ''))) for row in formatted_rows)) for h in headers]
    if title:
        print(f"\n{title}")
    # Print header
    header_line = ' | '.join(str(h).ljust(w) for h, w in zip(headers, col_widths))
    print(header_line)
    print('-+-'.join('-'*w for w in col_widths))
    # Print rows
    for row in formatted_rows:
        print(' | '.join(str(row.get(h, '')).ljust(w) for h, w in zip(headers, col_widths)))

def csv_report(data, headers=None):
    """
    Export statistics as a CSV string.
    data: list of dicts
    headers: optional list of column names
    """
    if not data:
        return ''
    if headers is None:
        headers = list(data[0].keys())
    output = io.StringIO()
    writer = csv.DictWriter(output, fieldnames=headers)
    writer.writeheader()
    for row in data:
        writer.writerow({h: row.get(h, '') for h in headers})
    return output.getvalue()

def generate_mermaid_structure(root_path, max_depth=5):
    """
    Generate a Mermaid diagram (flowchart TD) of the directory structure.
    """
    lines = ["flowchart TD"]
    node_id = 0
    node_map = {}
    def add_node(parent_id, path, depth):
        nonlocal node_id
        if depth > max_depth:
            return
        name = os.path.basename(path) or path
        this_id = f"n{node_id}"
        node_map[path] = this_id
        lines.append(f"    {this_id}[\"{name}\"]")
        if parent_id is not None:
            lines.append(f"    {parent_id} --> {this_id}")
        if os.path.isdir(path):
            try:
                for entry in sorted(os.listdir(path)):
                    if entry.startswith('.'):
                        continue
                    add_node(this_id, os.path.join(path, entry), depth+1)
            except Exception:
                pass
        node_id += 1
    add_node(None, root_path, 0)
    return '\n'.join(lines)

def generate_lang_card_svg(data, output_path, top_n=8):
    """
    Generate a beautiful SVG language stats card (like GitHub top-langs).
    data: list of dicts, each with 'ext' and 'total_lines' (from analyzer)
    output_path: SVG file path to write
    top_n: number of languages to show
    """
    # Sort by total_lines, take top_n
    sorted_data = sorted(data, key=lambda x: x['total_lines'], reverse=True)[:top_n]
    total = sum(x['total_lines'] for x in sorted_data)
    # Define a color palette (Material Design)
    palette = [
        '#1976D2', '#388E3C', '#FBC02D', '#D32F2F', '#7B1FA2', '#0288D1', '#F57C00', '#388E3C',
        '#C2185B', '#0097A7', '#FFA000', '#512DA8', '#00796B', '#303F9F', '#455A64', '#0288D1'
    ]
    width = 360
    height = 40 + 36 * len(sorted_data)
    bar_max_width = 180
    svg = [
        f'<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" fill="none" xmlns="http://www.w3.org/2000/svg">',
        '<style>\n',
        '  .title { font: 700 20px "Segoe UI", Arial, sans-serif; fill: #222; }\n',
        '  .lang { font: 600 15px "Segoe UI", Arial, sans-serif; fill: #333; }\n',
        '  .count { font: 400 13px "Segoe UI", Arial, sans-serif; fill: #666; }\n',
        '  .percent { font: 600 13px "Segoe UI", Arial, sans-serif; fill: #1976D2; }\n',
        '  .bar-bg { fill: #F0F4F8; }\n',
        '  .bar { rx: 6px; }\n',
        '  .card { filter: drop-shadow(0 2px 8px #0001); }\n',
        '</style>'
    ]
    svg.append(f'<rect class="card" x="0" y="0" width="{width}" height="{height}" rx="18" fill="#fff"/>')
    svg.append(f'<text x="24" y="32" class="title">Language Stats</text>')
    y0 = 56
    for i, item in enumerate(sorted_data):
        y = y0 + i * 36
        color = palette[i % len(palette)]
        percent = item['total_lines'] / total * 100 if total else 0
        bar_width = int(bar_max_width * (item['total_lines'] / sorted_data[0]['total_lines'])) if sorted_data[0]['total_lines'] else 0
        svg.append(f'<rect class="bar-bg" x="120" y="{y-16}" width="{bar_max_width}" height="20" rx="6"/>')
        svg.append(f'<rect class="bar" x="120" y="{y-16}" width="{bar_width}" height="20" fill="{color}" rx="6"/>')
        svg.append(f'<text x="32" y="{y}" class="lang">{item["ext"]}</text>')
        svg.append(f'<text x="{120+bar_max_width+24}" y="{y+1}" class="percent" alignment-baseline="middle">{percent:.1f}%</text>')
    svg.append('</svg>')
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(svg))


def generate_sustainability_badge_svg(score, output_path, label="Sustainability"):
    """
    Generate a modern SVG badge for sustainability/health score.
    Improved: better color, contrast, font, shadow, and gradient.
    """
    # Color by score (green/yellow/orange/red)
    if score >= 90:
        color = '#43A047'  # Green
        grad = '#66BB6A'
    elif score >= 75:
        color = '#FBC02D'  # Yellow
        grad = '#FFD54F'
    elif score >= 60:
        color = '#FFA726'  # Orange (brighter)
        grad = '#FFCC80'
    else:
        color = '#D32F2F'  # Red
        grad = '#E57373'
    width = 180
    height = 36
    label_w = 110
    value_w = width - label_w
    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<defs>',
        f'  <linearGradient id="score_grad" x1="0" y1="0" x2="0" y2="1">',
        f'    <stop offset="0%" stop-color="{grad}" stop-opacity="1"/>',
        f'    <stop offset="100%" stop-color="{color}" stop-opacity="1"/>',
        f'  </linearGradient>',
        '</defs>',
        f'<g>',
        f'  <rect x="0" y="0" width="{width}" height="{height}" rx="12" fill="#fff" opacity="0"/>',
        f'  <rect x="0" y="0" width="{label_w}" height="{height}" rx="12" fill="#555"/>',
        f'  <rect x="{label_w}" y="0" width="{value_w}" height="{height}" rx="12" fill="url(#score_grad)"/>',
        f'  <text x="{label_w//2}" y="22" text-anchor="middle" font-family="Segoe UI,Arial,sans-serif" font-size="15" font-weight="700" fill="#fff">{label}</text>',
        f'  <text x="{label_w+value_w//2}" y="22" text-anchor="middle" font-family="Segoe UI,Arial,sans-serif" font-size="16" font-weight="bold" fill="#fff">{score} / 100</text>',
        '</g>',
        '</svg>'
    ]
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(svg))

def get_project_name_from_git_or_dir(root_path):
    """
    Try to get project name from .git/config (github or remote url), else use directory name.
    """
    git_config = os.path.join(root_path, '.git', 'config')
    if os.path.exists(git_config):
        with open(git_config, 'r', encoding='utf-8', errors='ignore') as f:
            text = f.read()
        # Try to find github or remote url
        m = re.search(r'url = .*[/:]([^/\\]+/[^/\\]+?)(?:\\.git)?$', text, re.MULTILINE)
        if m:
            repo = m.group(1)
            # Use repo name only (after last /)
            name = repo.split('/')[-1]
            return name
    # fallback: use directory name
    return os.path.basename(os.path.abspath(root_path))

def generate_auto_readme(stats, health, contributors, hotspots, structure, badges=None, root_path='.'):
    """
    Auto-generate a README template with project structure, language stats, health score, and contributors.
    Optionally insert badges under the main title. Project name auto-detected.
    """
    lines = []
    project_name = get_project_name_from_git_or_dir(root_path)
    if project_name:
        lines.append(f'# Project {project_name} CodeState Summary')
    else:
        lines.append('# Project CodeState Summary')
    # Insert badges if provided
    if badges:
        lines.append('')
        lines.extend(badges)
    lines.append('')
    lines.append('## Project Structure')
    if structure:
        lines.append('```')
        lines.append(structure)
        lines.append('```')
    lines.append('')
    lines.append('## Language Statistics')
    lines.append('| Extension | Files | Lines | Comments | Functions | TODOs |')
    lines.append('|-----------|-------|-------|----------|-----------|-------|')
    for ext, info in stats.items():
        lines.append(f"| {ext} | {info['file_count']} | {info['total_lines']} | {info['comment_lines']} | {info['function_count']} | {info.get('todo_count', 0)} |")
    lines.append('')
    if health:
        lines.append('## Project Health')
        lines.append(f"- **Health Score:** {health['score']} / 100")
        lines.append(f"- **Average Comment Density:** {health['avg_comment_density']:.2%}")
        lines.append(f"- **Average Function Complexity:** {health['avg_complexity']:.2f}")
        lines.append(f"- **TODO/FIXME Count:** {health['todo_count']}")
        lines.append(f"- **Naming Violations:** {health['naming_violations']}")
        lines.append(f"- **Duplicate Code Blocks:** {health['duplicate_blocks']}")
        lines.append(f"- **Large Files:** {health['large_files']}")
        lines.append(f"- **Large Functions:** {health['large_functions']}")
        if health['suggestions']:
            lines.append('### Suggestions:')
            for s in health['suggestions']:
                lines.append(f"- {s}")
    if contributors:
        lines.append('## Top Contributors')
        lines.append('| author | file_count | line_count | commit_count | workload_percent |')
        lines.append('|--------|------------|------------|--------------|------------------|')
        for c in contributors:
            lines.append(
                f"| {c.get('author','')} | {c.get('file_count','')} | {c.get('line_count','')} | {c.get('commit_count','')} | {c.get('workload_percent','')} |"
            )
    if hotspots:
        lines.append('')
        lines.append('## Git Hotspots (Most Frequently Changed Files)')
        lines.append('| File | Commits |')
        lines.append('|------|---------|')
        for path, count in hotspots:
            lines.append(f"| {path} | {count} |")
    lines.append('')
    lines.append('---')
    lines.append('> Generated by [CodeState](https://github.com/HenryLok0/CodeState)')
    return '\n'.join(lines) 