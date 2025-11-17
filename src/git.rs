use serde::Serialize;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use zed_extension_api as zed;

const FIELD_DELIMITER: char = '\x1f';
const RECORD_DELIMITER: char = '\x1e';
const DEFAULT_LIMIT: usize = 400;
const MAX_LIMIT: usize = 2000;

/// Captures the git history for the provided worktree path.
pub struct GraphCollector {
    git_executable: PathBuf,
    worktree_root: PathBuf,
    env: Vec<(OsString, OsString)>,
}

impl GraphCollector {
    pub fn new(worktree: &zed::Worktree) -> Result<Self, GitGraphError> {
        let git_path = worktree
            .which("git")
            .ok_or(GitGraphError::GitBinaryMissing)?;
        let env = worktree
            .shell_env()
            .into_iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect::<Vec<_>>();

        Ok(Self {
            git_executable: PathBuf::from(git_path),
            worktree_root: PathBuf::from(worktree.root_path()),
            env,
        })
    }

    pub fn collect_graph(&self, limit: usize) -> Result<GitGraph, GitGraphError> {
        let sanitized_limit = limit.clamp(1, MAX_LIMIT);
        let fetch_limit = sanitized_limit.saturating_add(1);
        let output = run_git_log(
            &self.git_executable,
            &self.worktree_root,
            &self.env,
            fetch_limit,
        )?;
        let mut commits = parse_git_log(&output)?;
        let truncated = commits.len() >= fetch_limit;
        commits.truncate(sanitized_limit);
        Ok(GitGraph::new(commits, truncated))
    }
}

fn run_git_log(
    git_executable: &Path,
    worktree_root: &Path,
    env: &[(OsString, OsString)],
    limit: usize,
) -> Result<String, GitGraphError> {
    const FORMAT: &str = "%H%x1f%P%x1f%an%x1f%ae%x1f%cr%x1f%cI%x1f%ct%x1f%s%x1f%D%x1e";

    let mut command = Command::new(git_executable);
    command
        .arg("--no-pager")
        .arg("log")
        .arg("--all")
        .arg("--date-order")
        .arg("--decorate=full")
        .arg("--color=never")
        .arg(format!("--max-count={limit}"))
        .arg(format!("--pretty=format:{FORMAT}"))
        .current_dir(worktree_root);

    for (key, value) in env {
        command.env(key, value);
    }

    let output = command.output().map_err(GitGraphError::SpawnFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitGraphError::CommandFailed(stderr.trim().to_owned()));
    }

    String::from_utf8(output.stdout).map_err(GitGraphError::OutputEncoding)
}

fn parse_git_log(raw_output: &str) -> Result<Vec<CommitNode>, GitGraphError> {
    raw_output
        .split(RECORD_DELIMITER)
        .filter(|record| !record.trim().is_empty())
        .map(parse_record)
        .collect()
}

fn parse_record(record: &str) -> Result<CommitNode, GitGraphError> {
    let mut parts = record.split(FIELD_DELIMITER);
    let oid = parts
        .next()
        .ok_or_else(|| GitGraphError::Parse("missing commit hash".into()))?;
    let parents_part = parts.next().unwrap_or_default();
    let author = parts.next().unwrap_or_default();
    let author_email = parts.next().unwrap_or_default();
    let relative_time = parts.next().unwrap_or_default();
    let committed_at = parts.next().unwrap_or_default();
    let committed_timestamp = parts.next().unwrap_or_default();
    let summary = parts.next().unwrap_or_default();
    let decorations_raw = parts.next().unwrap_or_default();

    let parents = parents_part
        .split_whitespace()
        .filter(|parent| !parent.is_empty())
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let committed_timestamp = committed_timestamp
        .parse::<i64>()
        .map_err(|err| GitGraphError::Parse(format!("timestamp parse error: {err}")))?;

    Ok(CommitNode {
        oid: oid.to_owned(),
        short_oid: abbreviate_oid(oid),
        parents,
        author: author.to_owned(),
        author_email: author_email.to_owned(),
        relative_time: relative_time.to_owned(),
        committed_at: committed_at.to_owned(),
        committed_timestamp,
        summary: summary.to_owned(),
        decorations: CommitDecorations::from_raw(decorations_raw),
    })
}

fn abbreviate_oid(oid: &str) -> String {
    let abbrev_len = 8.min(oid.len());
    oid[..abbrev_len].to_owned()
}

#[derive(Debug, Serialize, Clone)]
pub struct GitGraph {
    pub commits: Vec<CommitNode>,
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

impl GitGraph {
    fn new(commits: Vec<CommitNode>, truncated: bool) -> Self {
        let mut edges = Vec::new();
        for commit in &commits {
            for parent in &commit.parents {
                edges.push(GraphEdge {
                    child: commit.oid.clone(),
                    parent: parent.clone(),
                });
            }
        }
        GitGraph {
            commits,
            edges,
            truncated,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct GraphEdge {
    pub child: String,
    pub parent: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub parents: Vec<String>,
    pub author: String,
    pub author_email: String,
    pub relative_time: String,
    pub committed_at: String,
    pub committed_timestamp: i64,
    pub summary: String,
    pub decorations: CommitDecorations,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct CommitDecorations {
    pub head: Option<String>,
    pub tags: Vec<String>,
    pub local_branches: Vec<String>,
    pub remote_branches: Vec<String>,
}

impl CommitDecorations {
    fn from_raw(raw: &str) -> Self {
        let mut decorations = CommitDecorations::default();
        for token in raw
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            if let Some(target) = token.strip_prefix("HEAD -> ") {
                decorations.head = Some(target.trim().to_owned());
            } else if let Some(tag) = token.strip_prefix("tag: ") {
                decorations.tags.push(tag.trim().to_owned());
            } else if token.contains('/') {
                decorations.remote_branches.push(token.to_owned());
            } else {
                decorations.local_branches.push(token.to_owned());
            }
        }
        decorations
    }
}

#[derive(Debug)]
pub enum GitGraphError {
    GitBinaryMissing,
    SpawnFailed(std::io::Error),
    CommandFailed(String),
    OutputEncoding(std::string::FromUtf8Error),
    Parse(String),
}

impl std::fmt::Display for GitGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitGraphError::GitBinaryMissing => write!(f, "git executable was not found on PATH"),
            GitGraphError::SpawnFailed(err) => write!(f, "failed to run git: {err}"),
            GitGraphError::CommandFailed(stderr) => {
                write!(f, "git log exited with an error: {stderr}")
            }
            GitGraphError::OutputEncoding(err) => {
                write!(f, "git output was not valid UTF-8: {err}")
            }
            GitGraphError::Parse(err) => write!(f, "failed to parse git output: {err}"),
        }
    }
}

impl std::error::Error for GitGraphError {}

#[derive(Debug, Clone)]
pub struct SlashCommandOptions {
    pub limit: usize,
}

impl SlashCommandOptions {
    pub fn from_args(args: &[String]) -> Result<Self, GitGraphError> {
        if args.is_empty() {
            return Ok(Self {
                limit: DEFAULT_LIMIT,
            });
        }
        let limit = usize::from_str(&args[0])
            .map_err(|err| GitGraphError::Parse(format!("invalid limit: {err}")))?;
        Ok(Self {
            limit: limit.clamp(1, MAX_LIMIT),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_git_log() {
        let raw = format!(
            "abc123{sep}{sep}Jane Doe{sep}jd@example.com{sep}2 days ago{sep}2024-05-01T10:00:00Z{sep}1714557600{sep}Initial commit{sep}HEAD -> main, origin/main{rec}",
            sep = FIELD_DELIMITER,
            rec = RECORD_DELIMITER
        );
        let parsed = parse_git_log(&raw).expect("should parse");
        assert_eq!(parsed.len(), 1);
        let node = &parsed[0];
        assert_eq!(node.oid, "abc123");
        assert_eq!(node.parents.len(), 0);
        assert_eq!(node.decorations.head.as_deref(), Some("main"));
        assert_eq!(node.decorations.remote_branches, &["origin/main"]);
    }
}
