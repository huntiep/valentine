pub mod network;

use {Context, Result};
use templates::RepoTmpl;
use types::*;

use git2::{ObjectType, Repository};
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

pub fn read<'a, 'b>(ctx: &'a Context, username: &'b str, repo_info: Repo)
    -> Result<RepoTmpl<'a, 'b>>
{
    let mut repo_name = repo_info.name.clone();
    if !repo_name.ends_with(".git") {
        repo_name += ".git";
    }

    let path = ctx.repo_dir.join(username).join(repo_name);
    let repo = Repository::open(path)?;
    let head = repo.head()?;
    // TODO
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;

    let mut readme = None;
    let mut items = Vec::with_capacity(tree.len());
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("Invalid filename").to_string();
        let kind = entry.kind().unwrap_or(ObjectType::Any);
        let name_lower = name.to_lowercase();

        if readme.is_none() && name_lower.starts_with("readme") &&
            kind == ObjectType::Blob
        {
            let obj = entry.to_object(&repo)?;
            let blob = obj.as_blob().unwrap();
            if blob.is_binary() {
                break;
            }
            let content = String::from_utf8(blob.content().to_vec()).ok();
            if content.is_none() {
                break;
            }
            let content = content.unwrap();
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

    let tmpl = RepoTmpl {
        name: &ctx.name,
        username: username,
        repo: repo_info,
        items: items,
        readme: readme,
    };
    Ok(tmpl)
}

fn parse_readme(readme: &str) -> String {
    let content = escape_html(readme);
    content.lines().fold(String::with_capacity(content.len()),
                         |acc, line| acc + line + "<br>")
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
