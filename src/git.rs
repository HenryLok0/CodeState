use git2::Repository;
use std::collections::HashMap;
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
            let diff = repo.diff_tree_to_tree(Some(&tree), Some(&prev), None)?;
            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    // Since we walk backwards in time, the first time we see a file changed
                    // is its latest modification.
                    file_authors.entry(path.to_string()).or_insert_with(|| author.clone());
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
        *author_counts.entry(author).or_insert(0) += 1;
    }
    
    let mut stats: Vec<ContributorStat> = author_counts
        .into_iter()
        .map(|(name, commits)| ContributorStat { name, commits })
        .collect();
        
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
