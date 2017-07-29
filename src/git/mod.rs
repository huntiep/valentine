pub mod network;

use {Context, Result};
use templates::RepoTmpl;
use types::*;

use git2::{self, ObjectType, Repository};
use hayaku::escape_html;
use pulldown_cmark;

use std::fs;
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

pub fn init<P, S>(ctx: &Context, username: P, repo_name: S) -> Result<()>
    where P: AsRef<Path>,
          S: Into<String>,
{
    let path = ctx.repo_dir.join(username);

    let mut repo_name = repo_name.into();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }
    let path = path.join(repo_name);

    Repository::init_bare(path)?;
    Ok(())
}

pub fn delete<P: AsRef<Path>>(ctx: &Context, username: P, repo_name: P) -> Result<()> {
    let path = ctx.repo_dir.join(username).join(repo_name);
    fs::remove_dir_all(path)?;
    Ok(())
}

pub fn read_tree<'repo>(repo: &Repository,
                        tree: &git2::Tree<'repo>,
                        handle_readme: bool)
    -> Result<(Vec<RepoItem>, Option<String>)>
{
    let mut items = Vec::with_capacity(tree.len());
    let mut readme = None;
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("Invalid filename").to_string();
        let kind = entry.kind().unwrap_or(ObjectType::Any);
        let name_lower = name.to_lowercase();

        if handle_readme && readme.is_none() && name_lower.starts_with("readme") &&
            kind == ObjectType::Blob
        {
            let content = match read_file(repo, &entry)? {
                Some(c) => c,
                None => break,
            };
            if name_lower.ends_with(".md") || name_lower.ends_with(".markdown") {
                let events = pulldown_cmark::Parser::new(&content);
                let mut buf = String::new();
                pulldown_cmark::html::push_html(&mut buf, events);
                readme = Some(buf);
            } else {
                readme = Some(parse_readme(&content));
            }
        }

        let item = RepoItem {
            name: name,
            obj_type: kind,
        };
        items.push(item);
    }
    Ok((items, readme))
}

pub fn read_file<'iter>(repo: &Repository, entry: &git2::TreeEntry<'iter>)
    -> Result<Option<String>>
{
    let obj = entry.to_object(repo)?;
    let blob = match obj.as_blob() {
        Some(b) => b,
        None => return Ok(None),
    };
    if blob.is_binary() {
        return Ok(None);
    }
    Ok(String::from_utf8(blob.content().to_vec()).ok())
}

pub fn read<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo)
    -> Result<RepoTmpl<'a, 'b>>
{
    let mut repo_name = repo_info.name.clone();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }

    let path = ctx.repo_dir.join(username).join(repo_name);
    let repo = Repository::open(path)?;
    if repo.is_empty()? {
        let tmpl = RepoTmpl {
            name: &ctx.name,
            username: username,
            repo: repo_info,
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
        items: items,
        readme: readme,
        empty: false,
    };
    Ok(tmpl)
}

fn parse_readme(readme: &str) -> String {
    let content = escape_html(readme);
    content.lines().fold(String::with_capacity(content.len()),
                         |acc, line| acc + line + "<br>")
}

pub fn read_src<'a, 'b>(ctx: &'a Context,
                        username: &'b str,
                        repo_info: &Repo,
                        branch: &str,
                        file: &str)
    -> Result<Option<RepoSrc>>
{
    let mut repo_name = repo_info.name.clone();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }

    let path = ctx.repo_dir.join(username).join(repo_name);
    let repo = Repository::open(path)?;
    let branch = match repo.find_branch(branch, git2::BranchType::Local) {
        Ok(b) => b,
        Err(e) => if e.code() == git2::ErrorCode::NotFound {
            return Ok(None);
        } else {
            return Err(::Error::from(e));
        },
    };
    let oid = branch.get().target().unwrap();
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let entry = match tree.get_path(Path::new(file)) {
        Ok(e) => e,
        Err(e) => if e.code() == git2::ErrorCode::NotFound {
            return Ok(None);
        } else {
            return Err(::Error::from(e));
        },
    };

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

pub fn log<P: AsRef<Path>>(ctx: &Context, username: P, repo_name: &str) -> Result<Vec<Commit>> {
    let mut repo_name = repo_name.to_string();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }

    let path = ctx.repo_dir.join(username).join(repo_name);
    let repo = Repository::open(path)?;

    let mut log = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    for id in revwalk {
        let id = id?;
        let commit = repo.find_commit(id)?;
        let item = Commit::new(commit)?;
        log.push(item);
    }
    Ok(log)
}
