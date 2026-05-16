mod scanner;
mod analyzer;
mod git;
mod visualizer;

use clap::Parser;
use std::time::Instant;
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target directory to analyze (default: current directory)
    #[arg(default_value = ".")]
    directory: String,

    /// Show git hotspots
    #[arg(long)]
    hotspot: bool,

    /// Generate a markdown project summary
    #[arg(long)]
    summary: bool,

    /// Show detailed statistics for each file
    #[arg(long)]
    details: bool,

    /// Export result as HTML table
    #[arg(long)]
    html: bool,

    /// Export result as Markdown table
    #[arg(long)]
    md: bool,

    /// Show only files with issues
    #[arg(long)]
    failures_only: bool,
    
    /// Use cache for faster analysis
    #[arg(long)]
    cache: bool,

    /// Compare two directories
    #[arg(long, num_args = 2)]
    compare: Option<Vec<String>>,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
    
    /// Allow other commands to not panic
    #[arg(short = 'x', long)]
    exclude: Option<Vec<String>>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("CodeState 2.0 (Rust Edition) initializing...");
    let start_time = Instant::now();

    println!("Scanning directory: {}", args.directory);
    
    // 1. File Scanning (Parallel)
    let scan_start = Instant::now();
    let file_stats = scanner::scan_directory(&args.directory, None);
    let aggregated = scanner::aggregate_by_ext(&file_stats);
    let scan_elapsed = scan_start.elapsed();
    println!("✓ File scanning completed in {:?}", scan_elapsed);
    
    // 2. Health Analysis (Parallel)
    let analysis_start = Instant::now();
    let paths: Vec<_> = file_stats.iter().map(|f| f.path.clone()).collect();
    let _analysis_stats = analyzer::analyze_files(&paths);
    let analysis_elapsed = analysis_start.elapsed();
    println!("✓ Complexity & Health analysis completed in {:?}", analysis_elapsed);

    // 3. Print Results
    visualizer::print_summary_table(&aggregated);

    // 4. Git Hotspots (Optional)
    if args.hotspot {
        println!("\nAnalyzing Git history for hotspots...");
        let git_start = Instant::now();
        match git::get_git_hotspots(&args.directory, 10) {
            Ok(hotspots) => {
                visualizer::print_git_hotspots(&hotspots);
                let git_elapsed = git_start.elapsed();
                println!("✓ Git analysis completed in {:?}", git_elapsed);
            },
            Err(e) => {
                println!("! Could not perform Git analysis: {}", e);
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("\nTotal analysis completed in {:?}", elapsed);
    
    Ok(())
}
