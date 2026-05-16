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

## Usage Scenarios

CodeState is designed to be instantly useful for different daily scenarios.

### 1. Basic Scan: How big is this project?
Just run the tool without any arguments to get a blazingly fast overview of the language distribution, total lines of code, and comment density.

```bash
codestate
```

### 2. Code Health Check: What needs refactoring?
Use the summary flag to get a detailed breakdown of codebase health, including average function complexity and TODO counts.

```bash
codestate --summary
```

### 3. Find Bug Hotspots: Where should I focus my code review?
CodeState integrates directly with Git to find "Hotspots"—files that are modified most frequently. Files with high complexity *and* high churn are prime candidates for bugs.

```bash
codestate --hotspot
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

## Available Commands

CodeState's new Rust engine focuses on speed and simplicity.

| Flag | Description |
|---|---|
| `(no flags)` | Scan current directory and print basic language distribution & LOC. |
| `--hotspot` | Analyze git history and show the files with the most commits. |
| `--summary` | Generate a health summary of the project. |
| `--details` | Show file-by-file metrics (complexity, TODOs, comments). |
| `--runall` | Run a self-test suite of all features. |

## Advantages over standard line counters

Why use CodeState when traditional counting tools already exist?

1. **Beyond Lines of Code:** Pure line counters tell you how big a project is, but CodeState tells you how *healthy* it is by calculating cyclomatic complexity, extracting TODOs, and tracking comment density.
2. **Git Hotspot Analysis:** CodeState natively reads your `.git` history to find files that are churning the most. It helps teams identify which legacy files are causing the most friction.
3. **CI/CD Ready & Beautiful UI:** With built-in GitHub Actions support and a gorgeous `comfy-table` terminal UI, it goes from local terminal to automated PR reviews with zero friction.

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

