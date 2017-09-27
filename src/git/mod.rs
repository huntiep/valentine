pub mod network;
mod util;

use {Context, Result};
use templates::RepoTmpl;
use types::*;
use self::util::*;

use git2::{self, ObjectType, Repository};

use std::fs;
use std::io::Write;
use std::path::{Path};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessMode {
    Read,
    Write,
}

impl AccessMode {
    pub fn new(verb: &str) -> Option<Self> {
        match verb {
            "git-upload-pack" | "git-upload-archive" => Some(AccessMode::Read),
            "git-receive-pack" => Some(AccessMode::Write),
            _ => None
        }
    }
}

pub fn create_user<P: AsRef<Path>>(ctx: &Context, username: P) -> Result<()> {
    let path = ctx.repo_dir.join(username);
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn delete_user<P: AsRef<Path>>(ctx: &Context, path: P) -> Result<()> {
    let mut root_dir = ctx.repo_dir.clone();
    root_dir.push(path);
    fs::remove_dir_all(root_dir)?;
    Ok(())
}

pub fn add_ssh_key(ctx: &Context, ssh_key: &SshKey) -> Result<()> {
    let mut ssh_dir = ctx.ssh_dir.clone();
    ssh_dir.push("authorized_keys");
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(ssh_dir)?;

    let key = format!("command=\"{} -c '{}' ssh key-{}\",\
no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty {}",
                          ctx.bin_path.display(), ctx.config_path.display(),
                          ssh_key.id, ssh_key.content);
    Ok(file.write_all(key.as_bytes())?)
}

pub fn init(ctx: &Context, username: &str, reponame: &str) -> Result<()> {
    let path = build_repo_path(ctx, username, reponame);
    Repository::init_bare(path)?;
    Ok(())
}

pub fn delete<P: AsRef<Path>>(ctx: &Context, username: P, repo_name: P) -> Result<()> {
    let path = ctx.repo_dir.join(username).join(repo_name);
    fs::remove_dir_all(path)?;
    Ok(())
}

pub fn read<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo)
    -> Result<RepoTmpl<'a, 'b>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;
    if repo.is_empty()? {
        let tmpl = RepoTmpl {
            name: &ctx.name,
            username: username,
            repo: repo_info,
            commit: "",
            items: Vec::new(),
            readme: None,
            empty: true,
        };
        return Ok(tmpl);
    }
    let head = repo.head()?;
    // TODO
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;

    let (items, readme) = read_tree(&repo, &tree, true)?;

    let tmpl = RepoTmpl {
        name: &ctx.name,
        username: username,
        repo: repo_info,
        commit: "master",
        items: items,
        readme: readme,
        empty: false,
    };
    Ok(tmpl)
}

pub fn read_src<'a, 'b>(ctx: &'a Context,
                        username: &'b str,
                        repo_info: &Repo,
                        name: &str,
                        file: &str)
    -> Result<Option<RepoSrc>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;
    let branch = catch_git!(repo.find_branch(name, git2::BranchType::Local),
                        git2::ErrorCode::NotFound,
                        None);
    let oid = branch.get().target().unwrap();
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let entry = catch_git!(tree.get_path(Path::new(file)), git2::ErrorCode::NotFound, None);

    match entry.kind() {
        Some(ObjectType::Tree) => {
            let e = entry.to_object(&repo)?;
            let e = if let Some(e) = e.as_tree() {
                e
            } else {
                return Ok(Some(RepoSrc::Error));
            };
            let (items, readme) = read_tree(&repo, &e, true)?;
            Ok(Some(RepoSrc::Dir { items, readme }))
        }
        Some(ObjectType::Blob) => {
            match read_file(&repo, &entry)? {
                Some(c) => Ok(Some(RepoSrc::File(c))),
                None => Ok(Some(RepoSrc::Error)),
            }
        }
        Some(ObjectType::Any) => return Ok(Some(RepoSrc::Error)),
        Some(ObjectType::Commit) => return Ok(Some(RepoSrc::Error)),
        Some(ObjectType::Tag) => return Ok(Some(RepoSrc::Error)),
        None => return Ok(None),
    }
}

pub fn log(ctx: &Context, username: &str, reponame: &str, branch: &str)
    -> Result<Option<Vec<Commit>>>
{
    let path = build_repo_path(ctx, username, reponame);
    let repo = Repository::open(path)?;
    let branch = catch_git!(repo.find_branch(branch, git2::BranchType::Local),
                        git2::ErrorCode::NotFound,
                        None);

    let oid = match branch.into_reference().target() {
        Some(o) => o,
        _ => return Ok(None),
    };

    let mut log = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push(oid)?;
    for id in revwalk {
        let id = id?;
        let commit = repo.find_commit(id)?;
        let item = Commit::new(commit)?;
        log.push(item);
    }
    Ok(Some(log))
}

pub fn commit<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo, commit: &'b str)
    -> Result<Option<RepoTmpl<'a, 'b>>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;
    let oid = git2::Oid::from_str(commit)?;
    let tree = catch_git!(repo.find_commit(oid), git2::ErrorCode::NotFound, None).tree()?;
    let (items, readme) = read_tree(&repo, &tree, true)?;
    let tmpl = RepoTmpl {
        name: &ctx.name,
        username: username,
        repo: repo_info,
        commit: commit,
        items: items,
        readme: readme,
        empty: false,
    };
    Ok(Some(tmpl))
}
