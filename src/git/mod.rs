use {Context, Result};

use git2::Repository;

use std::{fs, process};
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessMode {
    Read,
    Write,
}

impl AccessMode {
    pub fn new(verb: &str) -> Option<Self> {
        match verb {
            "git-upload-pack" => Some(AccessMode::Read),
            "git-upload-archive" => Some(AccessMode::Read),
            "git-receive-pack" => Some(AccessMode::Write),
            _ => None
        }
    }
}

pub fn create_user<P: AsRef<Path>>(ctx: &Context, username: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    fs::create_dir_all(root_dir)?;
    Ok(())
}

pub fn delete_user<P: AsRef<Path>>(ctx: &Context, path: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(path);
    fs::remove_dir_all(root_dir)?;
    Ok(())
}

pub fn init<P, S>(ctx: &Context, username: P, repo_name: S) -> Result<()>
    where P: AsRef<Path>,
          S: Into<String>,
{
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);

    let mut repo_name = repo_name.into();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }
    root_dir.push(repo_name);

    Repository::init_bare(root_dir)?;
    Ok(())
}

pub fn delete<P: AsRef<Path>>(ctx: &Context, username: P, repo_name: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    root_dir.push(repo_name);
    fs::remove_dir_all(root_dir)?;
    Ok(())
}

pub fn info(ctx: &Context, username: &str, repo_name: &str) -> Result<Vec<u8>> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    root_dir.push(repo_name);

    let command = process::Command::new("git-upload-pack")
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(root_dir)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .output()?;

    Ok(command.stdout)
}

pub fn pull(ctx: &Context, username: &str, repo_name: &str, body: &[u8]) -> Result<Vec<u8>> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(username);
    root_dir.push(repo_name);

    let mut command = process::Command::new("git-upload-pack")
        .arg("--stateless-rpc")
        .arg(root_dir)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()?;

    {
        let mut stdin = command.stdin.as_mut().unwrap();
        stdin.write_all(body)?;
    }
    let output = command.wait_with_output()?;

    if !output.status.success() {
        // TODO
    }

    Ok(output.stdout)
}
