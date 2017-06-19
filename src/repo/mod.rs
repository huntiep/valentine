use {Context, Result};

use git2::Repository;

use std::fs;
use std::path::Path;

pub fn create_user<P: AsRef<Path>>(ctx: &Context, path: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(path);
    fs::create_dir(root_dir)?;
    Ok(())
}

pub fn delete_user<P: AsRef<Path>>(ctx: &Context, path: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(path);
    fs::remove_dir_all(root_dir)?;
    Ok(())
}

pub fn init<P: AsRef<Path>>(ctx: &Context, path: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(path);
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
