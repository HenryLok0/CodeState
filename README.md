# CodeState CLI

[![Code Size](https://img.shields.io/github/languages/code-size/HenryLok0/CodeState?style=flat-square&logo=github)](https://github.com/HenryLok0/CodeState)
![PyPI - Version](https://img.shields.io/pypi/v/CodeState)

[![MIT License](https://img.shields.io/github/license/HenryLok0/CodeState?style=flat-square)](LICENSE)
[![Stars](https://img.shields.io/github/stars/HenryLok0/CodeState?style=flat-square)](https://github.com/HenryLok0/CodeState/stargazers)

A CLI tool that analyzes your local codebase and generates detailed statistics, such as lines of code per file type, function complexity, comment density, and more. It visualizes the data as ASCII charts or exports to JSON, HTML, or Markdown for further use. This tool is designed for developers who want quick insights into their project's structure without relying on external services.

## Installation

```bash
pip install codestate
```

Or for local development:

```bash
pip install .
```

## Usage

```bash
codestate [directory] [options]
```

### Options

| Option                | Description |
|-----------------------|-------------|
| `directory`           | Target directory to analyze (default: current directory) |
| `--exclude`           | Directories to exclude (space separated, e.g. --exclude .git venv node_modules) |
| `--ext`               | File extensions to include (e.g. --ext .py .js) |
| `--json`              | Export result as JSON |
| `--html`              | Export result as HTML table |
| `--md`                | Export result as Markdown table |
| `--details`           | Show detailed statistics for each file |
| `--dup`               | Show duplicate code blocks (5+ lines) |
| `--maxmin`            | Show file with most/least lines |
| `--authors`           | Show git main author and last modifier for each file |
| `--langdist`          | Show language (file extension) distribution as ASCII pie chart |
| `--naming`            | Check function/class naming conventions (PEP8, PascalCase) |
| `--tree`              | Show ASCII tree view of project structure |
| `--apidoc`            | Show API/function/class docstring summaries |
| `--warnsize`          | Warn for large files/functions (optionally specify file and function line thresholds, default 300/50) |
| `--regex`             | User-defined regex rules for custom code checks (space separated, enclose in quotes) |
| `--output`, `-o`      | Output file for HTML/Markdown/JSON/CSV/Markdown export |
| `--hotspot`           | Show most frequently changed files (git hotspots) |
| `--health`            | Show project health score and suggestions |
| `--groupdir`          | Show grouped statistics by top-level directory |
| `--groupext`          | Show grouped statistics by file extension |
| `--complexitymap`     | Show ASCII heatmap of file complexity |
| `--deadcode`          | Show unused (dead) functions/classes in Python files |
| `--ci`                | CI/CD mode: exit non-zero if major issues found |
| `--summary`           | Generate a markdown project summary (print or --output) |
| `--typestats`         | Show function parameter/type annotation statistics (Python) |
| `--security`          | Scan for common and advanced security issues (SSRF, RCE, SQLi, secrets, hardcoded credentials, etc.) |
| `--csv`               | Export summary statistics as CSV |
| `--details-csv`       | Export per-file details as CSV |
| `--groupdir-csv`      | Export grouped-by-directory stats as CSV |
| `--groupext-csv`      | Export grouped-by-extension stats as CSV |
| `--version`           | Show codestate version and exit |
| `--list-extensions`   | List all file extensions found in the project |
| `--size`   | Show each file's size in bytes as a table |
| `--trend`             | Show line count trend for a specific file (provide file path) |
| `--refactor-suggest`  | Show files/functions that are refactor candidates, with reasons |
| `--structure-mermaid` | Generate a Mermaid diagram of the project directory structure |
| `--openapi`           | Generate OpenAPI 3.0 JSON for Flask/FastAPI routes |
| `--style-check`       | Check code style: indentation, line length, trailing whitespace, EOF newline |
| `--multi`             | Analyze multiple root directories (monorepo support) |
| `--contributors`      | Show contributor statistics (file count, line count, commit count per author) |
| `--contributors-detail`      | Show detail contributor statistics |

## Examples

Analyze the current directory (excluding .git, venv, node_modules by default):
```bash
codestate
```

Analyze a specific directory and exclude build and dist folders:
```bash
codestate myproject --exclude build dist
```

Only analyze Python and JavaScript files:
```bash
codestate --ext .py .js
```

Export results as JSON:
```bash
codestate --json
```

Export results as HTML:
```bash
codestate --html
```

Export results as Markdown:
```bash
codestate --md
```

Show detailed statistics for each file:
```bash
codestate --details
```

Show duplicate code blocks (5+ lines):
```bash
codestate --dup
```

Show file with most/least lines:
```bash
codestate --maxmin
```

Show git main author and last modifier for each file:
```bash
codestate --authors
```

Show language distribution as ASCII pie chart:
```bash
codestate --langdist
```

Check function/class naming conventions:
```bash
codestate --naming
```

Show ASCII tree view of project structure:
```bash
codestate --tree
```

Show API/function/class docstring summaries:
```bash
codestate --apidoc
```

Warn for large files/functions (default 300/50 lines, or specify):
```bash
codestate --warnsize
codestate --warnsize 200 30
```

User-defined regex rules for custom code checks:
```bash
codestate --regex "TODO" "def [A-Z]"
codestate --regex "password" "eval\("
```

Export results as HTML to a file:
```bash
codestate --html --output report.html
```

Export results as Markdown to a file:
```bash
codestate --md --output report.md
```

Export results as JSON to a file:
```bash
codestate --json --output result.json
```

Show most frequently changed files (git hotspots):
```bash
codestate --hotspot
```

Show project health score and suggestions:
```bash
codestate --health
```

Show grouped statistics by top-level directory:
```bash
codestate --groupdir
```

Show grouped statistics by file extension:
```bash
codestate --groupext
```

Show ASCII heatmap of file complexity:
```bash
codestate --complexitymap
```

Show unused (dead) functions/classes in Python files:
```bash
codestate --deadcode
```

CI/CD mode: exit non-zero if major issues found:
```bash
codestate --ci
```

Generate a markdown project summary:
```bash
codestate --summary --output PROJECT_SUMMARY.md
```

Show function parameter/type annotation statistics:
```bash
codestate --typestats
```

Scan for common insecure patterns and secrets:
```bash
codestate --security
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