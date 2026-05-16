use git2::Repository;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, Context};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Hotspot {
    pub path: String,
    pub commits: usize,
}

pub struct ContributorStat {
    pub name: String,
    pub commits: usize,
}

pub struct ContributorDetail {
    pub name: String,
    pub commits: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub files_changed: HashSet<String>,
}

pub fn get_uncommitted_files(repo_path: &str) -> Result<HashSet<String>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
    
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true);
    
    let statuses = repo.statuses(Some(&mut opts))?;
    let mut uncommitted = HashSet::new();
    
    for entry in statuses.iter() {
        if let Some(path) = entry.path() {
            uncommitted.insert(path.to_string());
        }
    }
    
    Ok(uncommitted)
}

pub fn get_git_hotspots(repo_path: &str, limit: usize) -> Result<Vec<Hotspot>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk (is this a valid repo with commits?)")?;
    
    let mut file_counts: HashMap<String, usize> = HashMap::new();
    let mut previous_tree = None;
    
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        if let Some(prev) = previous_tree {
            let diff = repo.diff_tree_to_tree(Some(&tree), Some(&prev), None)?;
            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    *file_counts.entry(path.to_string()).or_insert(0) += 1;
                }
            }
        }
        previous_tree = Some(tree);
    }
    
    let mut hotspots: Vec<Hotspot> = file_counts
        .into_iter()
        .map(|(path, commits)| Hotspot { path, commits })
        .collect();
        
    hotspots.sort_by(|a, b| b.commits.cmp(&a.commits));
    hotspots.truncate(limit);
    
    Ok(hotspots)
}

pub fn get_file_authors(repo_path: &str) -> Result<HashMap<String, String>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk")?;
        
    let mut file_authors: HashMap<String, String> = HashMap::new();
    let mut previous_tree = None;
    
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        
        if let Some(prev) = previous_tree {
            if !author.to_lowercase().contains("cursor") {
                let diff = repo.diff_tree_to_tree(Some(&tree), Some(&prev), None)?;
                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                        // Since we walk backwards in time, the first time we see a file changed
                        // is its latest modification.
                        file_authors.entry(path.to_string()).or_insert_with(|| author.clone());
                    }
                }
            }
        }
        previous_tree = Some(tree);
    }
    
    Ok(file_authors)
}

pub fn get_contributor_stats(repo_path: &str) -> Result<Vec<ContributorStat>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk")?;
        
    let mut author_counts: HashMap<String, usize> = HashMap::new();
    
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        if author.to_lowercase().contains("cursor") {
            continue;
        }
        *author_counts.entry(author).or_insert(0) += 1;
    }
    
    let mut stats: Vec<ContributorStat> = author_counts
        .into_iter()
        .map(|(name, commits)| ContributorStat { name, commits })
        .collect();
        
    stats.sort_by(|a, b| b.commits.cmp(&a.commits));
    
    Ok(stats)
}

pub fn get_contributors_detail(repo_path: &str) -> Result<Vec<ContributorDetail>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk")?;
        
    let mut author_details: HashMap<String, ContributorDetail> = HashMap::new();
    
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        
        if author.to_lowercase().contains("cursor") {
            continue;
        }
        
        let detail = author_details.entry(author.clone()).or_insert(ContributorDetail {
            name: author.clone(),
            commits: 0,
            insertions: 0,
            deletions: 0,
            files_changed: HashSet::new(),
        });
        
        detail.commits += 1;

        if let Ok(tree) = commit.tree() {
            if let Ok(parent) = commit.parent(0) {
                if let Ok(parent_tree) = parent.tree() {
                    if let Ok(diff) = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None) {
                        if let Ok(stats) = diff.stats() {
                            detail.insertions += stats.insertions();
                            detail.deletions += stats.deletions();
                        }
                        for delta in diff.deltas() {
                            if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                                detail.files_changed.insert(path.to_string());
                            }
                        }
                    }
                }
            } else {
                // Initial commit
                if let Ok(diff) = repo.diff_tree_to_tree(None, Some(&tree), None) {
                    if let Ok(stats) = diff.stats() {
                        detail.insertions += stats.insertions();
                    }
                    for delta in diff.deltas() {
                        if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                            detail.files_changed.insert(path.to_string());
                        }
                    }
                }
            }
        }
    }
    
    let mut stats: Vec<ContributorDetail> = author_details.into_values().collect();
    stats.sort_by(|a, b| b.commits.cmp(&a.commits));
    
    Ok(stats)
}

pub fn get_recent_churn(repo_path: &str, days: u64, limit: usize) -> Result<Vec<Hotspot>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk")?;
        
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let cutoff_time = now - (days as i64 * 24 * 60 * 60);
    
    let mut file_counts: HashMap<String, usize> = HashMap::new();
    let mut previous_tree = None;
    
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        
        if commit.time().seconds() < cutoff_time {
            break;
        }
        
        let tree = commit.tree()?;
        
        if let Some(prev) = previous_tree {
            let diff = repo.diff_tree_to_tree(Some(&tree), Some(&prev), None)?;
            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    *file_counts.entry(path.to_string()).or_insert(0) += 1;
                }
            }
        }
        previous_tree = Some(tree);
    }
    
    let mut hotspots: Vec<Hotspot> = file_counts
        .into_iter()
        .map(|(path, commits)| Hotspot { path, commits })
        .collect();
        
    hotspots.sort_by(|a, b| b.commits.cmp(&a.commits));
    hotspots.truncate(limit);
    
    Ok(hotspots)
}

pub fn get_file_blame(repo_path: &str, file_path: &str) -> Result<HashMap<String, usize>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut blame_options = git2::BlameOptions::new();
    let blame = repo.blame_file(std::path::Path::new(file_path), Some(&mut blame_options))
        .with_context(|| format!("Failed to blame file '{}'", file_path))?;
        
    let mut author_lines: HashMap<String, usize> = HashMap::new();
    
    for hunk in blame.iter() {
        let author = hunk.final_signature().name().unwrap_or("Unknown").to_string();
        if author.to_lowercase().contains("cursor") {
            continue;
        }
        let lines = hunk.lines_in_hunk();
        *author_lines.entry(author).or_insert(0) += lines;
    }
    
    Ok(author_lines)
}

pub fn get_file_trend(repo_path: &str, file_path: Option<&str>, limit: usize) -> Result<Vec<(String, usize)>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at '{}'", repo_path))?;
        
    let mut revwalk = repo.revwalk()
        .context("Failed to create git revwalk")?;
        
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()
        .context("Failed to push HEAD to revwalk")?;
        
    let mut trend = Vec::new();
    let mut count = 0;
    
    for oid in revwalk {
        if count >= limit {
            break;
        }
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        if let Some(path) = file_path {
            if let Ok(entry) = tree.get_path(std::path::Path::new(path)) {
                if let Ok(blob) = repo.find_blob(entry.id()) {
                    let content = std::str::from_utf8(blob.content()).unwrap_or("");
                    let lines = content.lines().count();
                    let t = commit.time().seconds();
                    trend.push((format!("timestamp: {}", t), lines));
                    count += 1;
                }
            }
        } else {
            // Project total placeholder
            let t = commit.time().seconds();
            trend.push((format!("timestamp: {}", t), 0));
            count += 1;
        }
    }
    
    Ok(trend)
}
