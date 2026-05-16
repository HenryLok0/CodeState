use git2::{Repository, Error};
use std::collections::HashMap;
use anyhow::{Result, Context};

pub struct Hotspot {
    pub path: String,
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
