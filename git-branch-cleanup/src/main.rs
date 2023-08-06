use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug)]
enum GitPruneError {
    GitError(String),
    Other(String),
}

impl Display for GitPruneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitError(error) => write!(f, "Git Error: {error}"),
            Self::Other(error) => write!(f, "Other Error: {error}"),
        }
    }
}

impl Error for GitPruneError {}

fn main() -> Result<(), Box<dyn Error>> {
    let project_path = env::args().nth(1).ok_or("Missing argument: project path")?;
    let local_branches = get_local_branches(&project_path)?;
    local_branches
        .iter()
        .filter(|branch| is_branch_missing_on_origin(&project_path, branch).unwrap())
        .for_each(|missing_branch| {
            println!("Removing {missing_branch}");
            remove_branch(&project_path, missing_branch).unwrap_or_else(|err| {
                eprintln!("Error removing branch '{missing_branch}': {err}");
            });
        });
    Ok(())
}

fn get_local_branches(path: &str) -> Result<Vec<String>, GitPruneError> {
    let output = Command::new("git")
        .args(["branch"])
        .current_dir(path)
        .output()
        .map_err(|_| GitPruneError::Other("Failed to execute 'git branch' command".to_owned()))?;
    if output.status.success() {
        let branches = std::str::from_utf8(&output.stdout)
            .map_err(|_| {
                GitPruneError::Other("Invalid UTF-8 output from 'git branch command' ".to_owned())
            })?
            .lines()
            .map(|line| line.trim_start_matches('*').trim().to_owned())
            .collect::<Vec<_>>();
        Ok(branches)
    } else {
        Err(GitPruneError::GitError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

fn is_branch_missing_on_origin(path: &str, branch: &str) -> Result<bool, GitPruneError> {
    let output = Command::new("git")
        .args(["ls-remote", "--exit-code", "--heads", "origin", branch])
        .current_dir(path)
        .output()
        .map_err(|_| {
            GitPruneError::Other("Failed to execute 'git ls-remote' command".to_owned())
        })?;
    Ok(!output.status.success())
}

fn remove_branch(path: &str, branch: &str) -> Result<(), GitPruneError> {
    let output = Command::new("git")
        .args(["branch", "-d", branch])
        .current_dir(path)
        .output()
        .map_err(|_| {
            GitPruneError::Other("Failed to execute 'git branch -D' command".to_owned())
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(GitPruneError::GitError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}
