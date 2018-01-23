use {Context, Result};
use types::*;

use git2::{self, ObjectType, Repository};
use hayaku::escape_html;
use pulldown_cmark;

use std::path::PathBuf;

pub fn build_repo_path(ctx: &Context, username: &str, reponame: &str) -> PathBuf {
    let mut reponame = reponame.to_string();
    if !reponame.ends_with(".git") {
        reponame += ".git";
    }

    ctx.repo_dir.join(username).join(reponame)
}

fn parse_readme(readme: &str) -> String {
    let content = escape_html(readme);
    content.lines().fold(String::with_capacity(content.len()),
                         |acc, line| acc + line + "<br>")
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
