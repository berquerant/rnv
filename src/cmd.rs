use crate::entry::Entry;
use crate::git::Git;
use crate::iox;
use anyhow::{Context, Result, bail, ensure};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn default_renovate_id(repo_root: &Path) -> Result<String> {
    let Some(id) = repo_root.file_name() else {
        bail!("cannot get repo basename");
    };
    Ok(id.to_string_lossy().to_string())
}

fn generate_renovate(git: Git, renovate_id: &str, commit: Option<String>) -> Result<Entry> {
    ensure!(git.is_valid(), "repo is not a git repository");
    git.fetch()?;
    let current_commit = git
        .get_current_commit()
        .context("failed to get current commit")?;
    let commit = commit.unwrap_or(current_commit);
    let tag = git
        .get_same_or_newer_or_latest_tag(&commit)
        .with_context(|| format!("cannot infer tag from {}", commit))?;
    let dep_name = git.renovate_dep_name()?;
    let datasource = git.renovate_datasource()?;
    let entry = Entry::new(tag, datasource, dep_name, renovate_id.to_string());
    Ok(entry)
}

pub fn generate_renovate_setting(
    git: Git,
    renovate_id: &str,
    commit: Option<String>,
) -> Result<()> {
    let entry = generate_renovate(git, renovate_id, commit)?;
    println!("{}", entry);
    Ok(())
}

fn read_entries(lock_file: Option<PathBuf>) -> Result<Box<dyn Iterator<Item = Result<Entry>>>> {
    let lock_file = lock_file.as_deref();
    let lines = iox::read_lines(lock_file)?;
    let it = lines.map(|x| Entry::try_from(x).context("invalid entry"));
    Ok(Box::new(it))
}

fn lock<'a>(git: &Git<'a>, entry: &'a Entry, checkout: bool) -> Result<()> {
    ensure!(git.is_valid(), "repo is not a git repository");
    git.fetch()?;
    let tag = entry.get_value();
    let commit = git
        .get_commit_from_tag(&tag)
        .with_context(|| format!("cannot get commit from {}", tag))?;
    debug!("lock: {} => {}", entry, commit);
    println!("{}", commit);
    if checkout {
        git.checkout(&commit)
            .with_context(|| format!("failed to checkout to {}", commit))?;
    }
    Ok(())
}

pub fn get_lock(
    git: Git,
    renovate_id: &str,
    lock_file: Option<PathBuf>,
    checkout: bool,
) -> Result<()> {
    for entry in read_entries(lock_file)? {
        let entry = entry?;
        if entry.has_id(renovate_id) {
            return lock(&git, &entry, checkout);
        }
    }
    bail!("cannot find renovate entry: {}", renovate_id);
}

fn read_git_dirs<'a>(
    root: &'a Path,
    git_command: &'a str,
) -> Result<Box<dyn Iterator<Item = PathBuf> + 'a>> {
    let it = iox::read_dirs(root)?
        .filter(|x| {
            let p = &x.path();
            let g = Git::new(p, git_command);
            g.is_valid() && g.is_toplevel_dir()
        })
        .map(|x| x.path());
    Ok(Box::new(it))
}

pub fn batch_generate_renovate_settings(
    repo_root: &Path,
    git_command: &str,
    fail_fast: bool,
) -> Result<()> {
    let dirs = read_git_dirs(repo_root, git_command)?;
    for dir in dirs {
        info!("batch_generate_renovate_settings: {}", dir.display());
        match default_renovate_id(&dir) {
            Err(err) => {
                if fail_fast {
                    return Err(err);
                }
                warn!("ignore {} {}", dir.display(), err);
            }
            Ok(renovate_id) => {
                let git = Git::new(&dir, git_command);
                if let Err(err) = generate_renovate_setting(git, &renovate_id, None) {
                    if fail_fast {
                        return Err(err);
                    }
                    error!("failed {} {}", dir.display(), err);
                }
            }
        }
    }
    Ok(())
}

pub fn batch_get_lock(
    repo_root: &Path,
    git_command: &str,
    lock_file: Option<PathBuf>,
    fail_fast: bool,
    checkout: bool,
) -> Result<()> {
    let entries = read_entries(lock_file)?;
    let mut entry_map = HashMap::new();
    for entry in entries {
        match entry {
            Err(err) => {
                if fail_fast {
                    return Err(err);
                }
                warn!("{}", err);
            }
            Ok(e) => {
                entry_map.insert(e.get_id(), e);
            }
        }
    }
    let dirs = read_git_dirs(repo_root, git_command)?;
    for dir in dirs {
        info!("batch_get_lock: {}", dir.display());
        match default_renovate_id(&dir) {
            Err(err) => {
                if fail_fast {
                    return Err(err);
                }
                warn!("ignore {} {}", dir.display(), err);
            }
            Ok(renovate_id) => {
                let Some(entry) = entry_map.get(&renovate_id) else {
                    info!("no entries for {}", renovate_id);
                    continue;
                };
                let git = Git::new(&dir, git_command);
                if let Err(err) = lock(&git, entry, checkout) {
                    if fail_fast {
                        return Err(err);
                    }
                    warn!("{} {}", dir.display(), err);
                }
            }
        }
    }

    Ok(())
}
