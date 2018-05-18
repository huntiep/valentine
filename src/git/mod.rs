pub mod network;
mod util;

use {Context, Result};
use templates::{CommitTmpl, RefsTmpl, RepoTmpl};
use types::*;
use self::util::*;

use git2::{self, ObjectType, Repository};

use std::fs;
use std::io::{Read, Write};
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

pub fn delete_ssh_key(ctx: &Context, id: i32) -> Result<()> {
    let mut ssh_dir = ctx.ssh_dir.clone();
    ssh_dir.push("authorized_keys");
    let mut file = fs::File::open(&ssh_dir)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let buf = buf.lines().filter(|l| !l.contains(&format!("ssh key-{}", id))).collect::<String>();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(ssh_dir)?;
    Ok(file.write_all(buf.as_bytes())?)
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
            url: &ctx.url,
            ssh: &ctx.ssh,
            mount: &ctx.mount,
            username: username,
            repo: repo_info,
            branches: Vec::new(),
            tags: Vec::new(),
            commits: Vec::new(),
            readme: None,
            empty: true,
        };
        return Ok(tmpl);
    }
    let head = repo.head()?;
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let readme = read_readme(&repo, &tree)?;

    let branches_raw = repo.branches(None)?.take(10);
    let mut branches = Vec::new();
    for branch in branches_raw {
        if let Some(name) = branch?.0.name()? {
            branches.push(Branch { name: name.to_string() });
        }
    }

    let raw_tags = repo.tag_names(None)?;
    let raw_tags = raw_tags.iter().take(10);
    let mut tags = Vec::new();
    for tag in raw_tags {
        if let Some(name) = tag {
            tags.push(Tag { name: name.to_string() });
        }
    }

    let mut commits = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push(oid)?;
    for _ in 0..5 {
        if let Some(id) = revwalk.next() {
            let id = id?;
            let commit = repo.find_commit(id)?;
            let item = Commit::new(&commit)?;
            commits.push(item);
        } else {
            break;
        }
    }

    let tmpl = RepoTmpl {
        url: &ctx.url,
        ssh: &ctx.ssh,
        mount: &ctx.mount,
        username: username,
        repo: repo_info,
        branches: branches,
        tags: tags,
        commits: commits,
        readme: readme,
        empty: false,
    };
    Ok(tmpl)
}

pub fn refs<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo)
    -> Result<RefsTmpl<'a, 'b>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;
    let branches_raw: Vec<_> = repo.branches(None)?.take(5).collect();
    let mut branches = Vec::new();
    for branch in branches_raw {
        if let Some(name) = branch?.0.name()? {
            branches.push(Branch { name: name.to_string() });
        }
    }

    let mut tags = Vec::new();
    for tag in repo.tag_names(None)?.iter() {
        if let Some(name) = tag {
            tags.push(Tag { name: name.to_string() });
        }
    }

    Ok(RefsTmpl {
        mount: &ctx.mount,
        username: username,
        repo: repo_info,
        branches: branches,
        tags: tags,
    })
}

pub fn read_src<'a, 'b>(ctx: &'a Context,
                        username: &'b str,
                        repo_info: &Repo,
                        id: &str,
                        file: &str)
    -> Result<Option<RepoSrc>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;

    let commit = match get_commit(&repo, id)? {
        Some(r) => r,
        _ => return Ok(None),
    };
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
            let (items, readme) = read_tree(&repo, e)?;
            Ok(Some(RepoSrc::Dir { items, readme }))
        }
        Some(ObjectType::Blob) => {
            match read_file(&repo, &entry)? {
                Some(c) => Ok(Some(RepoSrc::File(c))),
                None => Ok(Some(RepoSrc::Error)),
            }
        }
        Some(ObjectType::Any) | Some(ObjectType::Commit) | Some(ObjectType::Tag) =>
            Ok(Some(RepoSrc::Error)),
        None => Ok(None),
    }
}

pub fn log(ctx: &Context, username: &str, reponame: &str, id: &str)
    -> Result<Option<(Vec<Commit>, Option<String>)>>
{
    let path = build_repo_path(ctx, username, reponame);
    let repo = Repository::open(path)?;
    let oid = if let Ok(oid) = git2::Oid::from_str(id) {
        oid
    } else {
        let reference = match get_ref(&repo, id)? {
            Some(r) => r,
            _ => return Ok(None),
        };

        match reference.target() {
            Some(o) => o,
            _ => return Ok(None),
        }
    };

    let mut log = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push(oid)?;
    let mut i = 0;
    while let Some(id) = revwalk.next() {
        let id = id?;
        let commit = repo.find_commit(id)?;
        let item = Commit::new(&commit)?;
        log.push(item);
        i += 1;
        if i == 50 {
            break;
        }
    }
    let next_page = if let Some(id) = revwalk.next() {
        Some(id?.to_string())
    } else {
        None
    };
    Ok(Some((log, next_page)))
}

pub fn commit<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo, id: &'b str)
    -> Result<Option<CommitTmpl<'a, 'b>>>
{
    let path = build_repo_path(ctx, username, &repo_info.name);
    let repo = Repository::open(path)?;
    let raw_commit = match get_commit(&repo, id)? {
        Some(r) => r,
        _ => return Ok(None),
    };

    let commit = Commit::new(&raw_commit)?;
    let tree = raw_commit.tree()?;
    let (items, readme) = read_tree(&repo, &tree)?;

    let tmpl = CommitTmpl {
        mount: &ctx.mount,
        username: username,
        repo: repo_info,
        refname: id,
        commit: commit,
        items: items,
        readme: readme,
    };
    Ok(Some(tmpl))
}
