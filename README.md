# CodeState

[MIT License](LICENSE)
[Stars](https://github.com/HenryLok0/CodeState/stargazers)

**Blazingly fast.**

Instant codebase insights in your terminal — fast, local, zero telemetry. CodeState provides extreme performance while keeping its unique advanced features like Git Hotspot analysis and Code Health tracking.

## Features

- **Written in Rust:** Blazingly fast multi-threaded file scanning powered by `rayon` and `ignore`.
- **Zero-Install:** Download the standalone binary and run it instantly. No dependencies required!
- **GitHub Action Ready:** Automate PR reviews and codebase health checks effortlessly.
- **Beautiful TUI:** Gorgeous terminal UI with colors and styled tables powered by `comfy-table`.
- Native Git Hotspot analysis via `git2-rs` for lighting fast history scans.
- Detailed stats: files, LOC, comments, functions, complexity.

## Installation

### Option 1: Zero-Install Binaries (Recommended)

Download the standalone binary from the [Releases page](https://github.com/HenryLok0/CodeState/releases). No Python installation required!

```bash
# Linux / macOS
curl -L https://github.com/HenryLok0/CodeState/releases/latest/download/codestate -o codestate
chmod +x codestate
./codestate

# Windows
# Download codestate.exe from Releases and run it directly
```

### Option 2: Build from Source

```bash
# Requires Rust and Cargo
git clone https://github.com/HenryLok0/CodeState.git
cd CodeState
cargo build --release
./target/release/codestate
```

## Quick Start

```bash
# Get a summary of the current repo
codestate --summary

# Export an HTML report
codestate --html --output report.html
```

## GitHub Action (CI/CD)

CodeState has built-in support for GitHub Actions! Automate codebase health checks and PR reviews by simply adding this to your `.github/workflows/pr.yml`:

```yaml
name: PR CodeState Check
on: [pull_request]

jobs:
  codestate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run CodeState Analysis
        uses: HenryLok0/CodeState@v1
        with:
          args: '--summary --failures-only'
          github_token: ${{ secrets.GITHUB_TOKEN }}
```

This action will automatically run CodeState and post a beautiful report as a PR comment.

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


| Option                               | Description                                                                                           |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------- |
| **[Basic Analysis & Filtering]**     |                                                                                                       |
| `directory`                          | Target directory to analyze (default: current directory)                                              |
| `--exclude`                          | Directories to exclude (e.g. --exclude .git venv node_modules)                                        |
| `--ext`                              | File extensions to include (e.g. --ext .py .js)                                                       |
| `--only-lang`                        | Only analyze specific file extensions, comma separated (e.g. py,js)                                   |
| `--top N`                            | Show only the top N files by lines or complexity                                                      |
| `--failures-only`                    | Show only files with issues (naming, size, complexity, etc.)                                          |
| `--regex`                            | User-defined regex rules for custom code checks (space separated, enclose in quotes)                  |
| `--file-age`                         | Show file creation and last modified time                                                             |
| `--uncommitted`                      | Show stats for files with uncommitted changes (git diff)                                              |
| `--size`                             | Show each file's size in bytes as a table                                                             |
| `--list-extensions`                  | List all file extensions found in the project with count and percentage                               |
| `--min-lines <N>`                    | Only show files with total lines >= N                                                                 |
| `--find <keyword/regex>`             | Find all lines matching a keyword or regex in the codebase                                            |
| `--cache`                            | Build and use cache for much faster repeated analysis (highly recommended for large codebases)        |
| `--cache-delete`                     | Delete all cache data in `.codestate` (force rebuild cache on next run)                               |
| **[Statistics & Detailed Analysis]** |                                                                                                       |
| `--details`                          | Show detailed statistics for each file                                                                |
| `--dup`                              | Show duplicate code blocks (5+ lines)                                                                 |
| `--maxmin`                           | Show file with most/least lines                                                                       |
| `--langdist`                         | Show language (file extension) distribution as ASCII pie chart                                        |
| `--complexitymap`                    | Show ASCII heatmap of file complexity                                                                 |
| `--complexity-graph`                 | Show an ASCII bar chart of file complexity                                                            |
| `--warnsize`                         | Warn for large files/functions (optionally specify file and function line thresholds, default 300/50) |
| `--naming`                           | Check function/class naming conventions (PEP8, PascalCase)                                            |
| `--apidoc`                           | Show API/function/class docstring summaries                                                           |
| `--deadcode`                         | Show unused (dead) functions/classes in Python files                                                  |
| `--typestats`                        | Show function parameter/type annotation statistics (Python)                                           |
| `--trend`                            | Show line count trend for a specific file                                                             |
| `--refactor-suggest`                 | Show files/functions that are refactor candidates, with reasons                                       |
| `--autofix-suggest`                  | Suggest auto-fix patches for naming, comments, and duplicate code                                     |
| `--refactor-map`                     | Show a table of files/functions that are refactor candidates                                          |
| `--complexity-threshold <value>`     | Set custom complexity threshold for warnings (**requires a value**, e.g. --complexity-threshold 5)    |
| `--open <file>`                      | Show detailed analysis for a single file                                                              |
| `--blame <file>`                     | Show git blame statistics for a file                                                                  |
| `--compare <dir1> <dir2>`            | Compare statistics between two directories                                                            |
| **[Output / Reports]**               |                                                                                                       |
| `--html`                             | Export result as HTML table                                                                           |
| `--md`                               | Export result as Markdown table                                                                       |
| `--json`                             | Export result as JSON                                                                                 |
| `--csv`                              | Export summary statistics as CSV                                                                      |
| `--excel`                            | Export summary statistics as Excel (.xlsx)                                                            |
| `--details-csv`                      | Export per-file details as CSV                                                                        |
| `--groupdir-csv`                     | Export grouped-by-directory stats as CSV                                                              |
| `--groupext-csv`                     | Export grouped-by-extension stats as CSV                                                              |
| `--test-coverage <coverage.xml>`     | Show test coverage analysis from a coverage.xml file                                                  |
| `--output`, `-o`                     | Output file for HTML/Markdown/JSON/CSV/Excel export                                                   |
| `--report-issues`                    | Export all detected issues (naming, size, complexity, etc.) as a markdown or JSON report              |
| **[Project Structure & Health]**     |                                                                                                       |
| `--tree`                             | Show ASCII tree view of project structure                                                             |
| `--structure-mermaid`                | Generate a Mermaid diagram of the project directory structure                                         |
| `--health`                           | Show project health score and suggestions                                                             |
| `--summary`                          | Generate a markdown project summary (print or --output)                                               |
| `--badge-sustainability`             | Output SVG sustainability/health badge                                                                |
| `--lang-card-svg`                    | Output SVG language stats card (like GitHub top-langs)                                                |
| **[Contributors / CI]**              |                                                                                                       |
| `--authors`                          | Show git main author and last modifier for each file                                                  |
| `--contributors`                     | Show contributor statistics (file count, line count, commit count per author)                         |
| `--contributors-detail`              | Show detailed contributor statistics                                                                  |
| `--hotspot`                          | Show most frequently changed files (git hotspots)                                                     |
| `--churn`                            | Show most changed files in the last N days (default 30)                                               |
| `--ci`                               | CI/CD mode: exit non-zero if major issues found                                                       |
| **[Automation / README / Badges]**   |                                                                                                       |
| `--badges`                           | Auto-detect and print project language/framework/license/CI badges for README                         |
| `--readme`                           | Auto-generate a README template based on analysis                                                     |
| **[Other]**                          |                                                                                                       |
| `--style-check`                      | Check code style: indentation, line length, trailing whitespace, EOF newline                          |
| `--openapi`                          | Generate OpenAPI 3.0 JSON for Flask/FastAPI routes                                                    |
| `--multi <dir1> [dir2 ...]`          | Analyze multiple root directories (monorepo support, **requires at least one directory**)             |
| `--version`                          | Show codestate version and exit                                                                       |


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

## Why CodeState?

- Instant understanding: go beyond LOC to highlight duplicates, complexity hotspots, refactor candidates, naming issues, and dead code — right in your terminal.
- Visual by default: ASCII pie/heatmap/bar charts make trends obvious during reviews, without leaving the CLI.
- History-aware decisions: git hotspots/churn help you prioritize the files that matter most.
- Team visibility: contributors and blame insights per file unlock ownership and onboarding context.
- CI-ready artifacts: export HTML/Markdown/JSON/CSV/Excel for reports, dashboards, and pipelines.
- Faster repeat runs: built-in caching and .gitignore support keep large repositories snappy over time.

Tip: Combine CodeState with a GitHub Action to post a compact Markdown summary on every PR.

## Star History

[Star History Chart](https://star-history.com/#HenryLok0/CodeState&Date)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Support

If you have questions or need help, please open an issue on GitHub.

Thank you to all contributors and the open-source community for your support.

---

## Troubleshooting

- Windows path/encoding quirks: run from a local folder (avoid syncing folders) and ensure UTF-8 console.
- Very large repos: run once with `--cache`, then subsequent commands will be much faster.

