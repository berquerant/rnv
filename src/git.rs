use anyhow::{Context, Result, bail, ensure};
use log::debug;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Git<'a> {
    dir: &'a Path,
    git: &'a str,
}

impl Git<'_> {
    pub fn new<'a>(dir: &'a Path, git: &'a str) -> Git<'a> {
        Git { dir, git }
    }
    fn cmd<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S> + Copy,
        S: AsRef<OsStr> + fmt::Display,
    {
        debug!(
            "call git on {} | {}{}",
            self.dir.display(),
            self.git,
            args.into_iter()
                .fold("".to_string(), |acc, x| { format!("{} {}", acc, x) }),
        );
        let got = Command::new(self.git)
            .args(args)
            .current_dir(self.dir)
            .stdin(Stdio::null())
            .output()
            .context("failed to call git command")?;
        ensure!(
            got.status.success(),
            "{}",
            String::from_utf8_lossy(&got.stderr)
        );
        let r = String::from_utf8_lossy(&got.stdout).trim().to_string();
        debug!("result git => {}", r);
        Ok(r)
    }
    pub fn is_valid(&self) -> bool {
        self.cmd(["status"]).is_ok()
    }
    pub fn is_toplevel_dir(&self) -> bool {
        let d = self.show_toplevel_dir();
        d.is_ok() && self.dir == d.unwrap()
    }
    pub fn show_toplevel_dir(&self) -> Result<PathBuf> {
        let got = self.cmd(["rev-parse", "--show-toplevel"])?;
        Ok(got.into())
    }
    pub fn fetch(&self) -> Result<()> {
        self.cmd(["fetch"]).map(|_| ())
    }
    pub fn checkout(self, commit: &str) -> Result<()> {
        self.cmd(["reset", "--hard", commit]).map(|_| ())
    }
    pub fn get_commit_from_tag(&self, tag: &str) -> Result<String> {
        self.cmd(["rev-parse", tag])
    }
    fn get_tag_from_commit(&self, commit: &str) -> Result<String> {
        self.cmd(["describe", "--tags", "--exact-match", commit])
    }
    fn get_latest_tag(&self, commit: &str) -> Result<String> {
        self.cmd(["describe", "--abbrev=0", "--tags", commit])
    }
    fn list_tags_order_by_creatordate_asc(&self) -> Result<Vec<String>> {
        let got = self.cmd(["tag", "-l", "--sort=creatordate", "--format=%(refname)"])?;
        let r: Vec<String> = got
            .lines()
            .map(|s| s.strip_prefix("refs/tags/").unwrap_or(s))
            .map(|s| s.to_string())
            .collect();
        Ok(r)
    }
    pub fn get_current_commit(&self) -> Result<String> {
        self.get_commit_from_tag("@")
    }
    pub fn get_same_or_newer_or_latest_tag(&self, commit: &str) -> Result<String> {
        if let Ok(tag) = self.get_tag_from_commit(commit) {
            return Ok(tag);
        }
        debug!(
            "get_same_or_newer_or_latest_tag: current commit is not a tag {}",
            commit
        );
        let latest_tag = self.get_latest_tag(commit)?;
        let tags = self.list_tags_order_by_creatordate_asc()?;
        let Some(index) = tags.iter().position(|x| *x == latest_tag) else {
            bail!("latest_tag {} exists but not found in all tags", latest_tag);
        };
        let next_tag_index = index + 1;
        let r = if next_tag_index < tags.len() {
            let t = tags[next_tag_index].clone();
            debug!("get_same_or_newer_or_latest_tag: found next tag {}", t);
            t
        } else {
            debug!(
                "get_same_or_newer_or_latest_tag: not found next tag of {}",
                latest_tag
            );
            latest_tag
        };
        Ok(r)
    }
    fn remote_origin_url(&self) -> Result<String> {
        self.cmd(["config", "--get", "remote.origin.url"])
    }
    pub fn renovate_dep_name(&self) -> Result<String> {
        let got = self.remote_origin_url()?;
        if got.starts_with("https://github.com/") {
            let s = got.strip_prefix("https://github.com/").unwrap();
            return Ok(s.strip_suffix(".git").unwrap_or(s).to_string());
        }
        if got.starts_with("git@github.com:") {
            let s = got.strip_prefix("git@github.com:").unwrap();
            return Ok(s.strip_suffix(".git").unwrap_or(s).to_string());
        }
        bail!("cannot infer renovate dep name from {}", got);
    }
    pub fn renovate_datasource(&self) -> Result<String> {
        let got = self.remote_origin_url()?;
        if got.starts_with("https://github.com/") || got.starts_with("git@github.com:") {
            return Ok("github-tags".to_string());
        }
        bail!("cannot infer renovate datasource from {}", got);
    }
}
