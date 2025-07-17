"""
CLI entry point for codestate.
"""
import sys
import argparse
import json
from .analyzer import Analyzer
from .visualizer import ascii_bar_chart, print_comment_density, html_report, markdown_report, ascii_pie_chart

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
    args = parser.parse_args()

    # Analyze codebase
    analyzer = Analyzer(args.directory, file_types=args.ext, exclude_dirs=args.exclude)
    stats = analyzer.analyze()

    # Prepare data for visualization
    data = []
    for ext, info in stats.items():
        item = {'ext': ext}
        item.update(info)
        data.append(item)

    if args.html:
        print(html_report(data, title='Code Statistics'))
    elif args.md:
        print(markdown_report(data, title='Code Statistics'))
    elif args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        ascii_bar_chart(data, value_key='total_lines', label_key='ext', title='Lines of Code per File Type')
        print_comment_density(data, label_key='ext')
        if args.langdist:
            ascii_pie_chart(data, value_key='file_count', label_key='ext', title='Language Distribution (by file count)')
        if args.details:
            print("\nFile Details:")
            for file_stat in analyzer.get_file_details():
                print(file_stat)
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

if __name__ == "__main__":
    main() 