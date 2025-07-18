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
    parser.add_argument('--readme', action='store_true', help='Auto-generate a README template based on analysis')
    parser.add_argument('--autofix-suggest', action='store_true', help='Suggest auto-fix patches for naming, comments, and duplicate code')
    parser.add_argument('--top', type=int, help='Show only the top N files by lines or complexity')
    parser.add_argument('--excel', nargs='?', const='codestate_report.xlsx', type=str, help='Export summary statistics as Excel (.xlsx)')
    parser.add_argument('--failures-only', action='store_true', help='Show only files with issues (naming, size, complexity, etc.)')
    parser.add_argument('--complexity-threshold', type=float, help='Set custom complexity threshold for warnings')
    parser.add_argument('--only-lang', type=str, help='Only analyze specific file extensions, comma separated (e.g. py,js)')
    parser.add_argument('--file-age', action='store_true', help='Show file creation and last modified time')
    parser.add_argument('--uncommitted', action='store_true', help='Show stats for files with uncommitted changes (git diff)')
    parser.add_argument('--all', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()

    # --all 隱藏自動測試分支
    if getattr(args, 'all', False):
        import subprocess
        import sys
        import os
        import platform
        # 根據平台決定丟棄檔案的路徑
        nullfile = 'NUL' if os.name == 'nt' else '/dev/null'
        commands = [
            ['--details'],
            ['--html', '--output', nullfile],
            ['--csv', '--output', nullfile],
            ['--excel', '--output', nullfile],
            ['--failures-only'],
            ['--top', '3'],
            ['--only-lang', 'py,js'],
            ['--file-age'],
            ['--uncommitted'],
            ['--summary', '--output', nullfile],
            ['--langdist'],
            ['--maxmin'],
            ['--contributors'],
            ['--contributors-detail'],
            ['--hotspot'],
            ['--style-check'],
            ['--security'],
            ['--deadcode'],
            ['--dup'],
            ['--apidoc'],
            ['--naming'],
            ['--warnsize'],
            ['--complexitymap'],
            ['--typestats'],
            ['--structure-mermaid', '--output', nullfile],
            ['--tree'],
            ['--openapi', '--output', nullfile],
            ['--version'],
            ['--list-extensions'],
        ]
        any_error = False
        for cmd in commands:
            try:
                result = subprocess.run(
                    [sys.executable, '-m', 'codestate.cli'] + cmd,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.PIPE,
                    cwd=args.directory if hasattr(args, 'directory') else '.',
                    text=True
                )
                if result.returncode != 0:
                    any_error = True
                    print(f"Error in command: {' '.join(cmd)}")
                    print(result.stderr)
            except Exception as e:
                any_error = True
                print(f"Exception in command: {' '.join(cmd)}: {e}")
        if not any_error:
            print('All options no error')
        sys.exit(0)

    # Analyze codebase
    regex_rules = args.regex if args.regex else None
    if args.multi:
        all_results = {}
        for d in args.multi:
            print(f'Analyzing {d} ...')
            analyzer = Analyzer(d, file_types=args.ext, exclude_dirs=args.exclude)
            # Show progress bar when analyzing files
            stats = analyzer.analyze(regex_rules=regex_rules)
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
    stats = analyzer.analyze(regex_rules=regex_rules)
    file_details = analyzer.get_file_details()
    data = file_details

    # --only-lang 過濾
    if args.only_lang:
        only_exts = [f'.{e.strip().lstrip(".")}' for e in args.only_lang.split(',') if e.strip()]
        data = [f for f in data if f['ext'] in only_exts]

    # --top N 過濾
    if args.top:
        data = sorted(data, key=lambda x: x.get('total_lines', 0), reverse=True)[:args.top]

    # --failures-only 過濾
    if args.failures_only:
        violations = analyzer.get_naming_violations()
        large_warn = analyzer.get_large_warnings(
            threshold_file=int(getattr(args, 'warnsize', [300])[0]) if getattr(args, 'warnsize', None) else 300,
            threshold_func=int(getattr(args, 'warnsize', [50, 50])[1]) if getattr(args, 'warnsize', None) and len(args.warnsize) > 1 else 50
        )
        failed_files = set()
        for v in violations:
            failed_files.add(v['file'])
        for w in large_warn['files']:
            failed_files.add(w['file'])
        for w in large_warn['functions']:
            failed_files.add(w['file'])
        # 複雜度過高
        complexity_threshold = args.complexity_threshold if args.complexity_threshold is not None else 3.0
        for f in data:
            if f.get('complexity', 0) > complexity_threshold:
                failed_files.add(f['path'])
        # 低註解密度
        for f in data:
            density = f.get('comment_lines', 0) / f['total_lines'] if f['total_lines'] else 0
            if density < 0.05:
                failed_files.add(f['path'])
        data = [f for f in data if f['path'] in failed_files]

    # 排除常見二進位/非程式碼檔案
    EXCLUDE_EXTS = {'.xlsx', '.xls', '.pdf', '.png', '.jpg', '.jpeg', '.gif', '.zip', '.tar', '.gz'}
    data = [f for f in data if f['ext'] not in EXCLUDE_EXTS]

    # --file-age 顯示
    if args.file_age:
        import os
        import datetime
        for f in data:
            try:
                stat = os.stat(os.path.abspath(f['path']))
                created = datetime.datetime.fromtimestamp(getattr(stat, 'st_ctime', 0)).strftime('%Y-%m-%d')
                modified = datetime.datetime.fromtimestamp(getattr(stat, 'st_mtime', 0)).strftime('%Y-%m-%d')
            except Exception:
                created = modified = 'N/A'
            f['created'] = created
            f['modified'] = modified
        headers = ["path", "created", "modified"]
        from .visualizer import print_table
        print_table(data, headers=headers, title='File Age Table:')
        return

    # --uncommitted 顯示
    if args.uncommitted:
        import os  # 修正 free variable bug
        import subprocess
        try:
            cmd = ['git', '-C', str(args.directory), 'diff', '--name-only']
            output = subprocess.check_output(cmd, encoding='utf-8', errors='ignore')
            changed_files = [line.strip() for line in output.splitlines() if line.strip()]
        except Exception:
            changed_files = []
        uncommitted = [f for f in data if os.path.relpath(f['path'], args.directory) in changed_files]
        # 顯示行數增減
        stats = []
        for f in uncommitted:
            try:
                diff_cmd = ['git', '-C', str(args.directory), 'diff', '--numstat', os.path.relpath(f['path'], args.directory)]
                diff_out = subprocess.check_output(diff_cmd, encoding='utf-8', errors='ignore')
                added, deleted, _ = diff_out.strip().split('\t') if diff_out.strip() else ('0','0','')
            except Exception:
                added, deleted = '0', '0'
            stats.append({'path': f['path'], 'lines_added': added, 'lines_deleted': deleted})
        from .visualizer import print_table
        print_table(stats, headers=["path", "lines_added", "lines_deleted"], title='Uncommitted changes:')
        return

    # --excel 匯出
    if args.excel:
        from .visualizer import export_excel_report
        output_path = args.excel if isinstance(args.excel, str) else 'codestate_report.xlsx'
        headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
        export_excel_report(data, output_path, headers=headers)
        print(f'Excel report written to {output_path}')
        return

    # 若有 --details 則詳細輸出
    if args.details:
        headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
        from .visualizer import print_table
        print_table(data, headers=headers, title='File Details:')
        return

    # 若有 --top 或 --failures-only 但沒指定 --details，預設也輸出表格
    if args.top or args.failures_only:
        headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
        from .visualizer import print_table
        print_table(data, headers=headers, title='Filtered File List:')
        return

    # 檔案分析完畢，進行輸出步驟
    if args.tree:
        from .visualizer import print_ascii_tree
        print('Project structure:')
        print_ascii_tree(args.directory)
        return

    # Prepare data for visualization
    data = []
    for ext, info in stats.items():
        item = {'ext': ext}
        item.update(info)
        data.append(item)

    if args.version:
        import sys
        print(f'codestate version {__version__}')
        sys.exit(0)
    if args.list_extensions:
        import sys
        exts = set()
        for file_path in analyzer._iter_files(args.directory):
            if file_path.suffix:
                exts.add(file_path.suffix)
        print('File extensions found in project:')
        for ext in sorted(exts):
            print(ext)
        sys.exit(0)

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
        from .visualizer import print_table
        file_details = analyzer.get_file_details_with_size()
        headers = ["path", "ext", "size", "total_lines", "comment_lines", "function_count"]
        print_table(file_details, headers=headers, title="File Sizes and Stats")
        return

    if args.trend:
        if not args.trend:
            print('Error: --trend requires a file path argument.')
            return
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
        from .visualizer import print_table
        stats = analyzer.get_contributor_stats()
        if not stats:
            print('No contributor statistics found (not a git repo or no data).')
        else:
            print('Contributor Statistics:')
            print_table(stats, headers=["author", "file_count", "line_count", "commit_count"], title=None)
        return

    if args.contributors_detail:
        from .visualizer import print_table
        stats = analyzer.get_contributor_stats()
        if not stats:
            print('No contributor statistics found (not a git repo or no data).')
        else:
            print('Contributor Detailed Statistics:')
            all_keys = set()
            for s in stats:
                all_keys.update(s.keys())
            headers = list(all_keys)
            preferred = ['author','file_count','line_count','commit_count','workload_percent','first_commit','last_commit','avg_lines_per_commit','main_exts','max_file_lines','active_days_last_30','added_lines','deleted_lines']
            headers = preferred + [k for k in headers if k not in preferred]
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
        from .visualizer import print_table
        print("\nFile with most lines:")
        print_table([mm['max_file']], title="Max File")
        print("File with least lines:")
        print_table([mm['min_file']], title="Min File")
        return
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
        import sys
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
        summary_md = generate_markdown_summary(data, analyzer.get_health_report(), hotspots)
        if args.output:
            abs_path = os.path.abspath(args.output)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(summary_md)
            print(f'Markdown project summary written to {abs_path}')
        else:
            print(summary_md)

    if args.readme:
        from .visualizer import generate_auto_readme, print_ascii_tree
        import io
        import contextlib
        import os
        # 保證所有參數都有預設值，避免 UnboundLocalError
        contributors = analyzer.get_contributor_stats() or []
        # 確保 contributors 有 workload_percent 欄位，並依 detail_workload_score 排序
        if contributors and ('workload_percent' not in contributors[0] or not contributors[0]['workload_percent']):
            total_score = sum(c.get('detail_workload_score', 0) for c in contributors)
            for c in contributors:
                if total_score > 0:
                    c['workload_percent'] = f"{(c.get('detail_workload_score', 0)/total_score*100):.1f}%"
                else:
                    c['workload_percent'] = '0.0%'
            contributors.sort(key=lambda c: c.get('detail_workload_score', 0), reverse=True)
        try:
            hotspots = analyzer.get_git_hotspots() or []
        except Exception:
            hotspots = []
        try:
            health = analyzer.get_health_report() or {}
        except Exception:
            health = {}
        # 專案結構
        buf = io.StringIO()
        try:
            with contextlib.redirect_stdout(buf):
                print_ascii_tree(args.directory)
            structure = buf.getvalue().strip()
        except Exception:
            structure = ''
        # --- badge 偵測（同 --badges）---
        badges = []
        exts = set()
        for file_path in analyzer._iter_files(args.directory):
            if file_path.suffix:
                exts.add(file_path.suffix.lower())
        lang_map = {
            '.py': 'Python', '.js': 'JavaScript', '.ts': 'TypeScript', '.java': 'Java', '.go': 'Go', '.rb': 'Ruby', '.php': 'PHP', '.cs': 'C#', '.cpp': 'C++', '.c': 'C', '.rs': 'Rust', '.kt': 'Kotlin', '.swift': 'Swift', '.m': 'Objective-C', '.scala': 'Scala', '.sh': 'Shell', '.pl': 'Perl', '.r': 'R', '.dart': 'Dart', '.jl': 'Julia', '.lua': 'Lua', '.hs': 'Haskell', '.html': 'HTML', '.css': 'CSS', '.json': 'JSON', '.yml': 'YAML', '.yaml': 'YAML', '.md': 'Markdown'
        }
        lang_count = {}
        for ext in exts:
            lang = lang_map.get(ext, ext.lstrip('.').capitalize())
            lang_count[lang] = lang_count.get(lang, 0) + 1
        main_lang = max(lang_count, key=lang_count.get) if lang_count else 'Unknown'
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
        # Detect GitHub repo
        github_repo = None
        git_config_path = os.path.join(args.directory, '.git', 'config')
        if os.path.exists(git_config_path):
            with open(git_config_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
            url = None
            for i, line in enumerate(lines):
                if '[remote "origin"]' in line:
                    for j in range(i+1, min(i+6, len(lines))):
                        if 'url =' in lines[j]:
                            url = lines[j].split('=',1)[1].strip()
                            break
                if url:
                    break
            if url:
                import re
                m = re.search(r'github.com[:/](.+?)(?:\.git)?$', url)
                if m:
                    github_repo = m.group(1)
        # 組 badge
        if github_repo:
            badges.append(f'[![Code Size](https://img.shields.io/github/languages/code-size/{github_repo}?style=flat-square&logo=github)](https://github.com/{github_repo})')
            badges.append(f'[![Stars](https://img.shields.io/github/stars/{github_repo}?style=flat-square)](https://github.com/{github_repo}/stargazers)')
        if main_lang != 'Unknown':
            badges.append(f'![Language](https://img.shields.io/badge/language-{main_lang}-blue?style=flat-square)')
        if license_type:
            badges.append(f'![License](https://img.shields.io/badge/license-{license_type}-yellow?style=flat-square)')
        readme_md = generate_auto_readme(
            stats, health, contributors, hotspots, structure,
            badges=badges, root_path=args.directory
        )
        # 決定輸出檔名：預設 README.md，若已存在則用 README.codestate.md
        if args.output:
            output_path = args.output
        else:
            default_readme = os.path.join(args.directory, 'README.md')
            if os.path.exists(default_readme):
                output_path = os.path.join(args.directory, 'README.codestate.md')
            else:
                output_path = default_readme
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(readme_md)
        print(f'Auto-generated README written to {output_path}')
        return
    if args.autofix_suggest:
        suggestions = analyzer.get_autofix_suggestions()
        print('\n'.join(suggestions))
        return

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
        import os
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
        ci = None
        gha_path = os.path.join(args.directory, '.github', 'workflows')
        if os.path.isdir(gha_path) and any(f.endswith('.yml') or f.endswith('.yaml') for f in os.listdir(gha_path)):
            ci = 'GitHub Actions'
        github_repo = None
        git_config_path = os.path.join(args.directory, '.git', 'config')
        if os.path.exists(git_config_path):
            with open(git_config_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
            url = None
            for i, line in enumerate(lines):
                if '[remote "origin"]' in line:
                    for j in range(i+1, min(i+6, len(lines))):
                        if 'url =' in lines[j]:
                            url = lines[j].split('=',1)[1].strip()
                            break
                if url:
                    break
            if url:
                import re
                m = re.search(r'github.com[:/](.+?)(?:\.git)?$', url)
                if m:
                    github_repo = m.group(1)
        print('\nRecommended README badges:')
        badge_md = []
        if github_repo:
            badge_md.append(f'[![Code Size](https://img.shields.io/github/languages/code-size/{github_repo}?style=flat-square&logo=github)](https://github.com/{github_repo})')
            badge_md.append(f'[![Stars](https://img.shields.io/github/stars/{github_repo}?style=flat-square)](https://github.com/{github_repo}/stargazers)')
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

    # fallback: if data has content and no other output options, print table
    if data and not any([
        args.details, args.html, args.csv, args.excel, args.top, args.failures_only, args.file_age, args.uncommitted,
        args.md, args.json, args.size, args.trend, args.refactor_suggest, args.structure_mermaid, args.openapi,
        args.style_check, args.contributors, args.contributors_detail, args.security, args.groupdir, args.groupext,
        args.groupdir_csv, args.groupext_csv, args.ci, args.summary, args.readme, args.autofix_suggest, args.lang_card_svg,
        args.badge_sustainability, args.badges, args.naming, args.apidoc, args.dup, args.maxmin, args.authors, args.langdist,
        args.complexitymap, args.deadcode, args.typestats, args.multi
    ]):
        headers = ["path", "ext", "total_lines", "comment_lines", "function_count", "complexity", "function_avg_length", "todo_count", "blank_lines", "comment_only_lines", "code_lines"]
        from .visualizer import print_table
        print_table(data, headers=headers, title='File List:')

if __name__ == "__main__":
    main() 