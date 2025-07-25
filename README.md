# CodeState

[![Code Size](https://img.shields.io/github/languages/code-size/HenryLok0/CodeState?style=flat-square&logo=github)](https://github.com/HenryLok0/CodeState)
![PyPI - Version](https://img.shields.io/pypi/v/CodeState)

[![MIT License](https://img.shields.io/github/license/HenryLok0/CodeState?style=flat-square)](LICENSE)
[![Stars](https://img.shields.io/github/stars/HenryLok0/CodeState?style=flat-square)](https://github.com/HenryLok0/CodeState/stargazers)

A CLI tool that analyzes your local codebase and generates detailed statistics, such as lines of code per file type, function complexity, comment density, and more. It visualizes the data as ASCII charts or exports to JSON, HTML, or Markdown for further use. This tool is designed for developers who want quick insights into their project's structure without relying on external services.

## Installation

```bash
pip install codestate
```

## Usage

```bash
# Basic usage
codestate [directory] [options]
```

### Cache Options (Highly recommended for large projects)
```bash
# Tip: For large projects, use --cache on the first run to build the cache for much faster repeated analysis
codestate --cache
# After the cache is built, subsequent queries (like --details, --html, --contributors, etc.) will automatically use the cache
# If there is no cache data, every command will scan the entire project again, which can be very slow for large projects.
codestate --details
# To rebuild the cache (e.g., after major refactoring or if the cache is outdated), delete the cache folder
codestate --cache-delete
```

## Options

| Option                  | Description |
|------------------------|-------------|
| **[Basic Analysis & Filtering]**    |             |
| `directory`            | Target directory to analyze (default: current directory) |
| `--exclude`            | Directories to exclude (e.g. --exclude .git venv node_modules) |
| `--ext`                | File extensions to include (e.g. --ext .py .js) |
| `--only-lang`          | Only analyze specific file extensions, comma separated (e.g. py,js) |
| `--top N`              | Show only the top N files by lines or complexity |
| `--failures-only`      | Show only files with issues (naming, size, complexity, etc.) |
| `--regex`              | User-defined regex rules for custom code checks (space separated, enclose in quotes) |
| `--file-age`           | Show file creation and last modified time |
| `--uncommitted`        | Show stats for files with uncommitted changes (git diff) |
| `--size`               | Show each file's size in bytes as a table |
| `--list-extensions`    | List all file extensions found in the project with count and percentage |
| `--min-lines <N>`      | Only show files with total lines >= N |
| `--find <keyword/regex>` | Find all lines matching a keyword or regex in the codebase |
| `--cache`              | Build and use cache for much faster repeated analysis (highly recommended for large codebases) |
| `--cache-delete`       | Delete all cache data in `.codestate
` (force rebuild cache on next run) |
| **[Statistics & Detailed Analysis]** |             |
| `--details`            | Show detailed statistics for each file |
| `--dup`                | Show duplicate code blocks (5+ lines) |
| `--maxmin`             | Show file with most/least lines |
| `--langdist`           | Show language (file extension) distribution as ASCII pie chart |
| `--complexitymap`      | Show ASCII heatmap of file complexity |
| `--complexity-graph`   | Show an ASCII bar chart of file complexity |
| `--warnsize`           | Warn for large files/functions (optionally specify file and function line thresholds, default 300/50) |
| `--naming`             | Check function/class naming conventions (PEP8, PascalCase) |
| `--apidoc`             | Show API/function/class docstring summaries |
| `--deadcode`           | Show unused (dead) functions/classes in Python files |
| `--typestats`          | Show function parameter/type annotation statistics (Python) |
| `--trend`              | Show line count trend for a specific file |
| `--refactor-suggest`   | Show files/functions that are refactor candidates, with reasons |
| `--autofix-suggest`    | Suggest auto-fix patches for naming, comments, and duplicate code |
| `--refactor-map`         | Show a table of files/functions that are refactor candidates |
| `--complexity-threshold <value>` | Set custom complexity threshold for warnings (**requires a value**, e.g. --complexity-threshold 5) |
| `--open <file>`        | Show detailed analysis for a single file |
| `--blame <file>`       | Show git blame statistics for a file |
| `--compare <dir1> <dir2>` | Compare statistics between two directories |
| **[Output / Reports]** |             |
| `--html`               | Export result as HTML table |
| `--md`                 | Export result as Markdown table |
| `--json`               | Export result as JSON |
| `--csv`                | Export summary statistics as CSV |
| `--excel`              | Export summary statistics as Excel (.xlsx) |
| `--details-csv`        | Export per-file details as CSV |
| `--groupdir-csv`       | Export grouped-by-directory stats as CSV |
| `--groupext-csv`       | Export grouped-by-extension stats as CSV |
| `--test-coverage <coverage.xml>` | Show test coverage analysis from a coverage.xml file |
| `--output`, `-o`       | Output file for HTML/Markdown/JSON/CSV/Excel export |
| `--report-issues`       | Export all detected issues (naming, size, complexity, etc.) as a markdown or JSON report |
| **[Project Structure & Health]** |             |
| `--tree`               | Show ASCII tree view of project structure |
| `--structure-mermaid`  | Generate a Mermaid diagram of the project directory structure |
| `--health`             | Show project health score and suggestions |
| `--summary`            | Generate a markdown project summary (print or --output) |
| `--badge-sustainability`| Output SVG sustainability/health badge |
| `--lang-card-svg`      | Output SVG language stats card (like GitHub top-langs) |
| **[Contributors / CI]** |             |
| `--authors`            | Show git main author and last modifier for each file |
| `--contributors`       | Show contributor statistics (file count, line count, commit count per author) |
| `--contributors-detail`| Show detailed contributor statistics |
| `--hotspot`            | Show most frequently changed files (git hotspots) |
| `--churn`              | Show most changed files in the last N days (default 30) |
| `--ci`                 | CI/CD mode: exit non-zero if major issues found |
| **[Automation / README / Badges]** |             |
| `--badges`             | Auto-detect and print project language/framework/license/CI badges for README |
| `--readme`             | Auto-generate a README template based on analysis |
| **[Other]**            |             |
| `--style-check`        | Check code style: indentation, line length, trailing whitespace, EOF newline |
| `--openapi`            | Generate OpenAPI 3.0 JSON for Flask/FastAPI routes |
| `--multi <dir1> [dir2 ...]`      | Analyze multiple root directories (monorepo support, **requires at least one directory**) |
| `--version`            | Show codestate version and exit |





## Examples

```bash
# Analyze the current directory (default)
codestate

# Analyze a specific directory and exclude build and dist folders
codestate myproject --exclude build dist

# Only analyze Python and JavaScript files
codestate --only-lang py,js

# Show only the top 5 largest files
codestate --top 5

# Show detailed statistics for each file
codestate --details

# Export results as HTML
codestate --html --output report.html

# Export results as CSV
codestate --csv --output report.csv

# Export results as Excel
codestate --excel --output report.xlsx

# Show only files with issues (naming, size, complexity, etc.)
codestate --failures-only

# Show file creation and last modified time
codestate --file-age

# Generate a markdown project summary
codestate --summary --output PROJECT_SUMMARY.md

# Set custom complexity threshold (requires a value)
codestate --complexity-threshold 5 --failures-only

# Analyze multiple directories (requires at least one directory)
codestate --multi src tests

# List all file extensions with count and percentage
codestate --list-extensions
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Support

If you have questions or need help, please open an issue on GitHub.

Thank you to all contributors and the open-source community for your support.