"""
Visualizer module for ASCII chart output.
"""

import os

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
    for item in data:
        label = str(item[label_key]).ljust(8)
        value = item[value_key]
        bar_len = int((value / max_value) * width) if max_value else 0
        bar = '█' * bar_len
        print(f"{label} | {bar} {value}")

def print_comment_density(data, label_key='ext'):
    """
    Print comment density as a percentage bar chart.
    """
    print("\nComment Density:")
    for item in data:
        label = str(item[label_key]).ljust(8)
        density = item.get('comment_density', 0)
        percent = int(density * 100)
        bar = '█' * (percent // 2)
        print(f"{label} | {bar} {percent}%")

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