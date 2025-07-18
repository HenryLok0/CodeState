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
codestate [directory] [options]
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
| `--list-extensions`    | List all file extensions found in the project |
| **[Statistics & Detailed Analysis]** |             |
| `--details`            | Show detailed statistics for each file |
| `--dup`                | Show duplicate code blocks (5+ lines) |
| `--maxmin`             | Show file with most/least lines |
| `--langdist`           | Show language (file extension) distribution as ASCII pie chart |
| `--complexitymap`      | Show ASCII heatmap of file complexity |
| `--warnsize`           | Warn for large files/functions (optionally specify file and function line thresholds, default 300/50) |
| `--naming`             | Check function/class naming conventions (PEP8, PascalCase) |
| `--apidoc`             | Show API/function/class docstring summaries |
| `--deadcode`           | Show unused (dead) functions/classes in Python files |
| `--typestats`          | Show function parameter/type annotation statistics (Python) |
| `--trend`              | Show line count trend for a specific file |
| `--refactor-suggest`   | Show files/functions that are refactor candidates, with reasons |
| `--autofix-suggest`    | Suggest auto-fix patches for naming, comments, and duplicate code |
| `--complexity-threshold`| Set custom complexity threshold for warnings |
| **[Output / Reports]** |             |
| `--html`               | Export result as HTML table |
| `--md`                 | Export result as Markdown table |
| `--json`               | Export result as JSON |
| `--csv`                | Export summary statistics as CSV |
| `--details-csv`        | Export per-file details as CSV |
| `--groupdir-csv`       | Export grouped-by-directory stats as CSV |
| `--groupext-csv`       | Export grouped-by-extension stats as CSV |
| `--excel`              | Export summary statistics as Excel (.xlsx) |
| `--output`, `-o`       | Output file for HTML/Markdown/JSON/CSV/Excel export |
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
| `--ci`                 | CI/CD mode: exit non-zero if major issues found |
| **[Automation / README / Badges]** |             |
| `--badges`             | Auto-detect and print project language/framework/license/CI badges for README |
| `--readme`             | Auto-generate a README template based on analysis |
| **[Other]**            |             |
| `--style-check`        | Check code style: indentation, line length, trailing whitespace, EOF newline |
| `--openapi`            | Generate OpenAPI 3.0 JSON for Flask/FastAPI routes |
| `--multi`              | Analyze multiple root directories (monorepo support) |
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