"""
CLI entry point for codestate.
"""
import sys
import argparse
import json
import os
from .analyzer import Analyzer
from .visualizer import ascii_bar_chart, print_comment_density, html_report, markdown_report, ascii_pie_chart, print_ascii_tree, ascii_complexity_heatmap, generate_markdown_summary, print_table, csv_report, generate_mermaid_structure# 新增 SVG 卡片/徽章函式
from .visualizer import generate_lang_card_svg, generate_sustainability_badge_svg
from . import __version__

def main():
    # Parse CLI arguments
    parser = argparse.ArgumentParser(description='CodeState: Codebase statistics CLI tool')
    parser.add_argument('directory', nargs='?', default='.', help='Target directory to analyze')
    parser.add_argument('--json', action='store_true', help='Export result as JSON')
    parser.add_argument('--html', action='store_true', help='Export result as HTML table')
    parser.add_argument('--md', action='store_true', help='Export result as Markdown table')
    parser.add_argument('--details', action='store_true', help='Show detailed statistics')
    parser.add_argument('--exclude', nargs='*', default=None, help='Directories to exclude (space separated)')
    parser.add_argument('--ext', nargs='*', default=None, help='File extensions to include (e.g. --ext .py .js)')
    parser.add_argument('--dup', action='store_true', help='Show duplicate code blocks (5+ lines)')
    parser.add_argument('--maxmin', action='store_true', help='Show file with most/least lines')
    parser.add_argument('--authors', action='store_true', help='Show git main author and last modifier for each file')
    parser.add_argument('--langdist', action='store_true', help='Show language (file extension) distribution as ASCII pie chart')
    parser.add_argument('--naming', action='store_true', help='Check function/class naming conventions (PEP8, PascalCase)')
    parser.add_argument('--tree', action='store_true', help='Show ASCII tree view of project structure')
    parser.add_argument('--apidoc', action='store_true', help='Show API/function/class docstring summaries')
    parser.add_argument('--warnsize', nargs='*', type=int, help='Warn for large files/functions (optionally specify file and function line thresholds, default 300/50)')
    parser.add_argument('--regex', nargs='+', help='User-defined regex rules for custom code checks (space separated, enclose in quotes)')
    parser.add_argument('--output', '-o', type=str, help='Output file for HTML/Markdown/JSON export')
    parser.add_argument('--hotspot', action='store_true', help='Show most frequently changed files (git hotspots)')
    parser.add_argument('--health', action='store_true', help='Show project health score and suggestions')
    parser.add_argument('--groupdir', action='store_true', help='Show grouped statistics by top-level directory')
    parser.add_argument('--groupext', action='store_true', help='Show grouped statistics by file extension')
    parser.add_argument('--complexitymap', action='store_true', help='Show ASCII heatmap of file complexity')
    parser.add_argument('--deadcode', action='store_true', help='Show unused (dead) functions/classes in Python files')
    parser.add_argument('--ci', action='store_true', help='CI/CD mode: exit non-zero if major issues found')
    parser.add_argument('--summary', action='store_true', help='Generate a markdown project summary (print or --output)')
    parser.add_argument('--typestats', action='store_true', help='Show function parameter/type annotation statistics (Python)')
    parser.add_argument('--security', action='store_true', help='Scan for common insecure patterns and secrets')
    parser.add_argument('--csv', action='store_true', help='Export summary statistics as CSV')
    parser.add_argument('--details-csv', action='store_true', help='Export per-file details as CSV')
    parser.add_argument('--groupdir-csv', action='store_true', help='Export grouped-by-directory stats as CSV')
    parser.add_argument('--groupext-csv', action='store_true', help='Export grouped-by-extension stats as CSV')
    parser.add_argument('--version', action='store_true', help='Show codestate version and exit')
    parser.add_argument('--list-extensions', action='store_true', help='List all file extensions found in the project')
    parser.add_argument('--size', action='store_true', help="Show each file's size in bytes as a table")
    parser.add_argument('--trend', type=str, help='Show line count trend for a specific file (provide file path)')
    parser.add_argument('--refactor-suggest', action='store_true', help='Show files/functions that are refactor candidates, with reasons')
    parser.add_argument('--structure-mermaid', action='store_true', help='Generate a Mermaid diagram of the project directory structure')
    parser.add_argument('--openapi', action='store_true', help='Generate OpenAPI 3.0 JSON for Flask/FastAPI routes')
    parser.add_argument('--style-check', action='store_true', help='Check code style: indentation, line length, trailing whitespace, EOF newline')
    parser.add_argument('--multi', nargs='+', help='Analyze multiple root directories (monorepo support)')
    parser.add_argument('--contributors', action='store_true', help='Show contributor statistics (file count, line count, commit count per author)')
    parser.add_argument('--contributors-detail', action='store_true', help='Show detailed contributor statistics (all available fields)')
    parser.add_argument('--lang-card-svg', nargs='?', const='codestate_langs.svg', type=str, help='Output SVG language stats card (like GitHub top-langs)')
    parser.add_argument('--badge-sustainability', nargs='?', const='codestate_sustainability.svg', type=str, help='Output SVG sustainability/health badge')
    parser.add_argument('--badges', action='store_true', help='Auto-detect and print project language/framework/license/CI badges for README')
    args = parser.parse_args()

    # Analyze codebase
    regex_rules = args.regex if args.regex else None
    if args.multi:
        all_results = {}
        for d in args.multi:
            print(f'Analyzing {d} ...')
            analyzer = Analyzer(d, file_types=args.ext, exclude_dirs=args.exclude)
            # Show progress bar when analyzing files
            stats = analyzer.analyze(regex_rules=regex_rules, show_progress=True)
            all_results[d] = stats
            data = []
            for ext, info in stats.items():
                item = {'ext': ext}
                item.update(info)
                data.append(item)
            print(f'--- {d} ---')
            ascii_bar_chart(data, value_key='total_lines', label_key='ext', title='Lines of Code per File Type')
            print_comment_density(data, label_key='ext')
        if args.output:
            import json
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(json.dumps(all_results, indent=2, ensure_ascii=False))
            print(f'Multi-project JSON written to {abs_path}')
        return

    analyzer = Analyzer(args.directory, file_types=args.ext, exclude_dirs=args.exclude)
    # 預先取得所有檔案數量
    if analyzer.file_types is None:
        files = [file_path for file_path in analyzer._iter_files(analyzer.root_dir) if file_path.suffix]
    else:
        files = [file_path for file_path in analyzer._iter_files(analyzer.root_dir) if file_path.suffix in analyzer.file_types]
    output_steps = 0
    if args.tree:
        output_steps += 1
    if args.badges:
        output_steps += 1
    # 你可以根據其他 CLI 參數再加 output_steps
    total_steps = len(files) + output_steps
    from tqdm import tqdm
    pbar = tqdm(total=total_steps, desc="總進度")
    def file_callback(_):
        pbar.update(1)
    stats = analyzer.analyze(regex_rules=regex_rules, file_callback=file_callback)
    # 檔案分析完畢，進行輸出步驟
    if args.tree:
        print('Project structure:')
        print_ascii_tree(args.directory)
        pbar.update(1)
    if args.badges:
        # ... badges 輸出 ...
        pbar.update(1)
    # 其他輸出步驟同理
    pbar.close()

    # Prepare data for visualization
    data = []
    for ext, info in stats.items():
        item = {'ext': ext}
        item.update(info)
        data.append(item)

    if args.version:
        print(f'codestate version {__version__}')
        sys.exit(0)
    if args.list_extensions:
        # Scan all files and print unique extensions
        exts = set()
        for file_path in analyzer._iter_files(args.directory):
            if file_path.suffix:
                exts.add(file_path.suffix)
        print('File extensions found in project:')
        for ext in sorted(exts):
            print(ext)
        sys.exit(0)

    if args.tree:
        print('Project structure:')
        print_ascii_tree(args.directory)

    if args.html:
        result = html_report(data, title='Code Statistics')
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(result)
            print(f'HTML report written to {abs_path}')
        else:
            print(result)
    elif args.md:
        result = markdown_report(data, title='Code Statistics')
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(result)
            print(f'Markdown report written to {abs_path}')
        else:
            print(result)
    elif args.json:
        result = json.dumps(data, indent=2, ensure_ascii=False)
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(result)
            print(f'JSON report written to {abs_path}')
        else:
            print(result)

    if args.size:
        file_details = analyzer.get_file_details_with_size()
        headers = ["path", "ext", "size", "total_lines", "comment_lines", "function_count"]
        print_table(file_details, headers=headers, title="File Sizes and Stats")
        return

    if args.trend:
        trend = analyzer.get_file_trend(args.trend)
        if not trend:
            print(f'No trend data found for {args.trend}.')
        else:
            print(f'Line count trend for {args.trend}:')
            print('Date       | Lines | Commit')
            print('-----------+-------+------------------------------------------')
            for t in trend:
                lines = t["lines"] if t["lines"] is not None else "-"
                print(f'{t["date"]:10} | {str(lines):5} | {t["commit"]}')
        return

    if args.refactor_suggest:
        suggestions = analyzer.get_refactor_suggestions()
        if not suggestions:
            print('No refactor suggestions found. All files look good!')
        else:
            print('Refactor Suggestions:')
            for s in suggestions:
                print(f"{s['path']} (lines: {s['total_lines']}, complexity: {s['complexity']}, avg_func_len: {s['function_avg_length']:.1f}, comment_density: {s['comment_density']:.1%}, TODOs: {s['todo_count']})")
                for reason in s['reasons']:
                    print(f"  - {reason}")
        return

    if args.structure_mermaid:
        mermaid = generate_mermaid_structure(args.directory)
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(mermaid)
            print(f'Mermaid structure diagram written to {abs_path}')
        else:
            print(mermaid)
        return

    if args.openapi:
        spec = analyzer.get_openapi_spec()
        if not spec or not spec.get('paths'):
            print('No Flask/FastAPI routes found for OpenAPI generation.')
        else:
            import json
            result = json.dumps(spec, indent=2, ensure_ascii=False)
            if args.output:
                abs_path = os.path.abspath(args.output)
                with open(args.output, 'w', encoding='utf-8') as f:
                    f.write(result)
                print(f'OpenAPI JSON written to {abs_path}')
            else:
                print(result)
        return

    if args.style_check:
        issues = analyzer.get_style_issues()
        if not issues:
            print('No code style issues found!')
        else:
            print('Code Style Issues:')
            for i in issues:
                print(f"{i['file']} (line {i['line']}): {i['type']} - {i['desc']}")
        return

    if args.contributors:
        stats = analyzer.get_contributor_stats()
        if not stats:
            print('No contributor statistics found (not a git repo or no data).')
        else:
            print('Contributor Statistics:')
            print_table(stats, headers=["author", "file_count", "line_count", "commit_count"], title=None)
        return

    if args.contributors_detail:
        stats = analyzer.get_contributor_stats()
        if not stats:
            print('No contributor statistics found (not a git repo or no data).')
        else:
            print('Contributor Detailed Statistics:')
            # Dynamically get all keys for headers
            all_keys = set()
            for s in stats:
                all_keys.update(s.keys())
            headers = list(all_keys)
            # Sort common fields first
            preferred = ['author','file_count','line_count','commit_count','workload_percent','first_commit','last_commit','avg_lines_per_commit','main_exts','max_file_lines','active_days_last_30','added_lines','deleted_lines']
            headers = preferred + [k for k in headers if k not in preferred]
            # Calculate weighted workload_percent using custom weights
            weights = {
                'line_count': 0.25,
                'commit_count': 0.20,
                'added_lines': 0.25,
                'deleted_lines': 0.15,
                'active_days_last_30': 0.05,
                'max_file_lines': 0.10
            }
            numeric_fields = list(weights.keys())
            for s in stats:
                score = 0
                for f in numeric_fields:
                    try:
                        score += float(s.get(f,0)) * weights[f]
                    except Exception:
                        pass
                s['_detail_workload_score'] = score
            total_score = sum(s['_detail_workload_score'] for s in stats)
            for s in stats:
                if total_score > 0:
                    s['workload_percent'] = f"{(s['_detail_workload_score']/total_score*100):.1f}%"
                else:
                    s['workload_percent'] = '0.0%'
            # Sort by detail workload score
            stats = sorted(stats, key=lambda s: s['_detail_workload_score'], reverse=True)
            print_table(stats, headers=headers, title=None)
        return

    if args.security:
        # Output both basic and advanced security issues
        issues = analyzer.get_security_issues()
        adv_issues = analyzer.get_advanced_security_issues()
        if not issues and not adv_issues:
            print('No security issues detected.')
        else:
            if issues:
                print('\n[Basic Security Issues]')
                for i in issues:
                    print(f"{i['file']} (line {i['line']}): {i['desc']}\n    {i['content']}")
                    if 'note' in i:
                        print(f"    Note: {i['note']}")
            if adv_issues:
                print('\n[Advanced Security Issues]')
                for i in adv_issues:
                    print(f"{i['file']} (line {i['line']}): {i['desc']}\n    {i['content']}")
                    if 'note' in i:
                        print(f"    Note: {i['note']}")
        return

    # Only show default bar chart if no arguments (just 'codestate')
    if len(sys.argv) == 1:
        ascii_bar_chart(data, value_key='total_lines', label_key='ext', title='Lines of Code per File Type')
        print_comment_density(data, label_key='ext')

    if args.langdist:
        ascii_pie_chart(data, value_key='file_count', label_key='ext', title='Language Distribution (by file count)')
    if args.details:
        print("\nFile Details:")
        file_details = analyzer.get_file_details()
        if file_details:
            headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
            print_table(file_details, headers=headers, title=None)
        else:
            print("No file details available.")
    if args.dup:
        dups = analyzer.get_duplicates()
        print(f"\nDuplicate code blocks (block size >= 5 lines, found {len(dups)} groups):")
        for group in dups:
            print("---")
            for path, line, block in group:
                print(f"File: {path}, Start line: {line}")
                print(block)
    if args.maxmin:
        mm = analyzer.get_max_min_stats()
        print("\nFile with most lines:")
        print(mm['max_file'])
        print("File with least lines:")
        print(mm['min_file'])
    if args.authors:
        authors = analyzer.get_git_authors()
        if authors is None:
            print("No .git directory found or not a git repo.")
        else:
            print("\nGit authorship info:")
            for path, info in authors.items():
                print(f"{path}: main_author={info['main_author']}, last_author={info['last_author']}")
    if args.naming:
        violations = analyzer.get_naming_violations()
        if not violations:
            print('All function/class names follow conventions!')
        else:
            print('\nNaming convention violations:')
            for v in violations:
                print(f"{v['type']} '{v['name']}' in {v['file']} (line {v['line']}): should be {v['rule']}")
    if args.apidoc:
        api_docs = analyzer.get_api_doc_summaries()
        if not api_docs:
            print('No API docstrings found.')
        else:
            print('\nAPI/function/class docstring summaries:')
            for d in api_docs:
                print(f"{d['type']} '{d['name']}' in {d['file']} (line {d['line']}):\n{d['docstring']}\n")
    if args.warnsize is not None:
        threshold_file = args.warnsize[0] if len(args.warnsize) > 0 else 300
        threshold_func = args.warnsize[1] if len(args.warnsize) > 1 else 50
        warnings = analyzer.get_large_warnings(threshold_file, threshold_func)
        if not warnings['files'] and not warnings['functions']:
            print(f'No files or functions exceed the thresholds ({threshold_file} lines for files, {threshold_func} for functions).')
        else:
            if warnings['files']:
                print(f'\nLarge files (>{threshold_file} lines):')
                for w in warnings['files']:
                    print(f"{w['file']} - {w['lines']} lines")
            if warnings['functions']:
                print(f'\nLarge functions (>{threshold_func} lines):')
                for w in warnings['functions']:
                    print(f"{w['function']} in {w['file']} (line {w['line']}) - {w['lines']} lines")
    if regex_rules:
        matches = analyzer.get_regex_matches()
        if not matches:
            print('No matches found for custom regex rules.')
        else:
            print('\nCustom regex matches:')
            for m in matches:
                print(f"{m['file']} (line {m['line']}): [{m['rule']}] {m['content']}")
    if args.hotspot:
        hotspots = analyzer.get_git_hotspots(top_n=10)
        if not hotspots:
            print('No git hotspot data found (not a git repo or no commits).')
        else:
            print('\nGit Hotspots (most frequently changed files):')
            for path, count in hotspots:
                print(f'{path}: {count} commits')
    if args.health:
        report = analyzer.get_health_report()
        if not report:
            print('No health report available.')
        else:
            print(f"\nProject Health Score: {report['score']} / 100")
            print(f"Average comment density: {report['avg_comment_density']:.2%}")
            print(f"Average function complexity: {report['avg_complexity']:.2f}")
            print(f"TODO/FIXME count: {report['todo_count']}")
            print(f"Naming violations: {report['naming_violations']}")
            print(f"Duplicate code blocks: {report['duplicate_blocks']}")
            print(f"Large files: {report['large_files']}")
            print(f"Large functions: {report['large_functions']}")
            if report['suggestions']:
                print("\nSuggestions:")
                for s in report['suggestions']:
                    print(f"- {s}")
            else:
                print("\nNo major issues detected. Great job!")
    if args.complexitymap:
        ascii_complexity_heatmap(analyzer.get_file_details(), title='File Complexity Heatmap')
    if args.deadcode:
        unused = analyzer.get_unused_defs()
        if not unused:
            print('No unused (dead) functions/classes found.')
        else:
            print('\nUnused (dead) functions/classes:')
            for d in unused:
                print(f"{d['type']} '{d['name']}' in {d['file']} (line {d['line']})")
    if args.typestats:
        stats = analyzer.get_api_param_type_stats()
        print('\nFunction Parameter/Type Annotation Statistics:')
        print(f"Total functions: {stats.get('total_functions', 0)}")
        print(f"Total parameters: {stats.get('total_parameters', 0)}")
        print(f"Annotated parameters: {stats.get('annotated_parameters', 0)}")
        print(f"Annotated returns: {stats.get('annotated_returns', 0)}")
        print(f"Parameter annotation coverage: {stats.get('param_annotation_coverage', 0):.2%}")
        print(f"Return annotation coverage: {stats.get('return_annotation_coverage', 0):.2%}")
    if args.groupdir:
        grouped = analyzer.get_grouped_stats(by='dir')
        print('\nGrouped statistics by top-level directory:')
        rows = []
        for d, stats in grouped.items():
            row = {'dir': d}
            row.update(stats)
            rows.append(row)
        print_table(rows, headers=["dir", "file_count", "total_lines", "comment_lines", "function_count"])
    if args.groupext:
        grouped = analyzer.get_grouped_stats(by='ext')
        print('\nGrouped statistics by file extension:')
        rows = []
        for ext, stats in grouped.items():
            row = {'ext': ext}
            row.update(stats)
            rows.append(row)
        print_table(rows, headers=["ext", "file_count", "total_lines", "comment_lines", "function_count"])

    if args.csv:
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8', newline='') as f:
                f.write(csv_report(data))
            print(f'CSV report written to {abs_path}')
        else:
            print(csv_report(data))
    if args.details_csv:
        file_details = analyzer.get_file_details()
        headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
        csv_str = csv_report(file_details, headers=headers)
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8', newline='') as f:
                f.write(csv_str)
            print(f'Details CSV written to {abs_path}')
        else:
            print(csv_str)
    if args.groupdir_csv:
        grouped = analyzer.get_grouped_stats(by='dir')
        rows = []
        for d, stats in grouped.items():
            row = {'dir': d}
            row.update(stats)
            rows.append(row)
        csv_str = csv_report(rows, headers=["dir", "file_count", "total_lines", "comment_lines", "function_count"])
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8', newline='') as f:
                f.write(csv_str)
            print(f'Grouped-by-directory CSV written to {abs_path}')
        else:
            print(csv_str)
    if args.groupext_csv:
        grouped = analyzer.get_grouped_stats(by='ext')
        rows = []
        for ext, stats in grouped.items():
            row = {'ext': ext}
            row.update(stats)
            rows.append(row)
        csv_str = csv_report(rows, headers=["ext", "file_count", "total_lines", "comment_lines", "function_count"])
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8', newline='') as f:
                f.write(csv_str)
            print(f'Grouped-by-extension CSV written to {abs_path}')
        else:
            print(csv_str)

    if args.ci:
        # Criteria: health score < 80, or any naming violations, large files/functions, or dead code
        report = analyzer.get_health_report()
        naming_violations = analyzer.get_naming_violations()
        large_warn = analyzer.get_large_warnings()
        deadcode = analyzer.get_unused_defs()
        fail = False
        reasons = []
        if report and report['score'] < 80:
            fail = True
            reasons.append(f"Health score too low: {report['score']}")
        if naming_violations:
            fail = True
            reasons.append(f"Naming violations: {len(naming_violations)}")
        if large_warn['files'] or large_warn['functions']:
            fail = True
            reasons.append(f"Large files: {len(large_warn['files'])}, Large functions: {len(large_warn['functions'])}")
        if deadcode:
            fail = True
            reasons.append(f"Dead code: {len(deadcode)} unused functions/classes")
        if fail:
            print("\nCI/CD check failed due to:")
            for r in reasons:
                print(f"- {r}")
            sys.exit(1)
        else:
            print("\nCI/CD check passed. No major issues detected.")
            sys.exit(0)

    if args.summary:
        hotspots = analyzer.get_git_hotspots(top_n=10)
        summary_md = generate_markdown_summary(stats, analyzer.get_health_report(), hotspots)
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(summary_md)
            print(f'Markdown project summary written to {abs_path}')
        else:
            print(summary_md)

    # 語言統計 SVG 卡片
    if args.lang_card_svg:
        # Prepare language stats data (by extension)
        lang_data = []
        for ext, info in stats.items():
            lang_data.append({'ext': ext, 'total_lines': info['total_lines']})
        # Generate SVG card
        output_path = args.lang_card_svg if isinstance(args.lang_card_svg, str) else 'codestate_langs.svg'
        generate_lang_card_svg(lang_data, output_path)
        print(f'Language stats SVG card written to {os.path.abspath(output_path)}')
        return
    # 可持續性/健康徽章 SVG
    if args.badge_sustainability:
        # Get health score from analyzer
        health = analyzer.get_health_report()
        score = health['score'] if health else 0
        output_path = args.badge_sustainability if isinstance(args.badge_sustainability, str) else 'codestate_sustainability.svg'
        generate_sustainability_badge_svg(score, output_path)
        print(f'Sustainability badge SVG written to {os.path.abspath(output_path)}')
        return

    if args.badges:
        # Auto-detect language
        exts = set()
        for file_path in analyzer._iter_files(args.directory):
            if file_path.suffix:
                exts.add(file_path.suffix.lower())
        lang_map = {
            '.py': 'Python', '.js': 'JavaScript', '.ts': 'TypeScript', '.java': 'Java', '.go': 'Go', '.rb': 'Ruby', '.php': 'PHP', '.cs': 'C%23', '.cpp': 'C%2B%2B', '.c': 'C', '.rs': 'Rust', '.kt': 'Kotlin', '.swift': 'Swift', '.m': 'Objective-C', '.scala': 'Scala', '.sh': 'Shell', '.pl': 'Perl', '.r': 'R', '.dart': 'Dart', '.jl': 'Julia', '.lua': 'Lua', '.hs': 'Haskell', '.html': 'HTML', '.css': 'CSS', '.json': 'JSON', '.yml': 'YAML', '.yaml': 'YAML', '.md': 'Markdown'
        }
        lang_count = {}
        for ext in exts:
            lang = lang_map.get(ext, ext.lstrip('.').capitalize())
            lang_count[lang] = lang_count.get(lang, 0) + 1
        main_lang = max(lang_count, key=lang_count.get) if lang_count else 'Unknown'
        # Detect framework (simple: look for requirements.txt, package.json, etc.)
        framework = None
        req_path = os.path.join(args.directory, 'requirements.txt')
        if os.path.exists(req_path):
            with open(req_path, 'r', encoding='utf-8', errors='ignore') as f:
                reqs = f.read().lower()
            if 'django' in reqs:
                framework = 'Django'
            elif 'flask' in reqs:
                framework = 'Flask'
            elif 'fastapi' in reqs:
                framework = 'FastAPI'
            elif 'torch' in reqs or 'tensorflow' in reqs:
                framework = 'ML'
        pkg_path = os.path.join(args.directory, 'package.json')
        if os.path.exists(pkg_path):
            import json as _json
            with open(pkg_path, 'r', encoding='utf-8', errors='ignore') as f:
                pkg = _json.load(f)
            deps = str(pkg.get('dependencies', {})).lower() + str(pkg.get('devDependencies', {})).lower()
            if 'react' in deps:
                framework = 'React'
            elif 'vue' in deps:
                framework = 'Vue.js'
            elif 'next' in deps:
                framework = 'Next.js'
            elif 'nuxt' in deps:
                framework = 'Nuxt.js'
        # Detect license
        license_type = None
        for lic_file in ['LICENSE', 'LICENSE.txt', 'LICENSE.md', 'license', 'license.txt']:
            lic_path = os.path.join(args.directory, lic_file)
            if os.path.exists(lic_path):
                with open(lic_path, 'r', encoding='utf-8', errors='ignore') as f:
                    lic_text = f.read().lower()
                if 'mit license' in lic_text:
                    license_type = 'MIT'
                elif 'apache license' in lic_text:
                    license_type = 'Apache'
                elif 'gpl' in lic_text:
                    license_type = 'GPL'
                elif 'bsd' in lic_text:
                    license_type = 'BSD'
                elif 'mozilla public license' in lic_text:
                    license_type = 'MPL'
                else:
                    license_type = 'Custom'
                break
        # Detect CI (GitHub Actions)
        ci = None
        gha_path = os.path.join(args.directory, '.github', 'workflows')
        if os.path.isdir(gha_path) and any(f.endswith('.yml') or f.endswith('.yaml') for f in os.listdir(gha_path)):
            ci = 'GitHub Actions'
        # Print badges
        print('\nRecommended README badges:')
        badge_md = []
        if main_lang != 'Unknown':
            badge_md.append(f'![Language](https://img.shields.io/badge/language-{main_lang}-blue?style=flat-square)')
        if framework:
            badge_md.append(f'![Framework](https://img.shields.io/badge/framework-{framework}-brightgreen?style=flat-square)')
        if license_type:
            badge_md.append(f'![License](https://img.shields.io/badge/license-{license_type}-yellow?style=flat-square)')
        if ci:
            badge_md.append(f'![CI](https://img.shields.io/badge/CI-{ci}-blue?style=flat-square)')
        if badge_md:
            for b in badge_md:
                print(b)
            print('\nYou can copy and paste the above Markdown into your README.')
        else:
            print('No badges detected.')
        return

if __name__ == "__main__":
    main() 