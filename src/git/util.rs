use {Context, Result};
use types::*;

use git2::{self, ObjectType, Repository};
use pulldown_cmark;

use std::path::PathBuf;

pub fn build_repo_path(ctx: &Context, username: &str, reponame: &str) -> PathBuf {
    let mut reponame = reponame.to_string();
    if !reponame.ends_with(".git") {
        reponame += ".git";
    }

    ctx.repo_dir.join(username).join(reponame)
}

pub fn read_readme<'repo>(repo: &Repository, tree: &git2::Tree<'repo>) -> Result<Option<String>> {
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("Invalid filename").to_string();
        let kind = entry.kind().unwrap_or(ObjectType::Any);
        let name_lower = name.to_lowercase();

        if name_lower.starts_with("readme") && kind == ObjectType::Blob {
            match handle_readme(repo, entry, name_lower)? {
                Some(r) => return Ok(Some(r)),
                _ => {},
            }
        }
    }

    Ok(None)
}

pub fn read_tree<'repo>(repo: &Repository, tree: &git2::Tree<'repo>)
    -> Result<(Vec<RepoItem>, Option<String>)>
{
    let mut items = Vec::with_capacity(tree.len());
    let mut readme = None;
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("Invalid filename").to_string();
        let kind = entry.kind().unwrap_or(ObjectType::Any);
        let name_lower = name.to_lowercase();

        if readme.is_none() && name_lower.starts_with("readme") &&
            kind == ObjectType::Blob
        {
            match handle_readme(repo, entry, name_lower)? {
                Some(r) => readme = Some(r),
                _ => {},
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

fn handle_readme(repo: &Repository, entry: git2::TreeEntry, name_lower: String)
    -> Result<Option<String>>
{
    let content = match read_file(repo, &entry)? {
        Some(c) => c,
        None => return Ok(None),
    };
    if name_lower.ends_with(".md") || name_lower.ends_with(".markdown") {
        let events = pulldown_cmark::Parser::new(&content);
        let mut buf = String::new();
        pulldown_cmark::html::push_html(&mut buf, events);
        return Ok(Some(buf));
    } else {
        return Ok(Some(parse_readme(&content)));
    }
}

fn parse_readme(readme: &str) -> String {
    let content = html_escape::encode_text(readme);
    content.lines().fold(String::with_capacity(content.len()),
                         |acc, line| acc + line + "<br>")
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

pub fn get_ref<'a>(repo: &'a Repository, name: &str) -> Result<Option<git2::Reference<'a>>> {
    // HEAD must be handled specially
    if name == "HEAD" {
        Ok(Some(repo.head()?))
    } else {
        // Refs are of the form refs/{heads|tags}/{name}. This glob supports
        // searching both branches and tags. There may be more types of refs
        // that this also supports, not sure.
        let mut refs = repo.references_glob(&format!("*/{}", name))?;
        let mut refs = refs.names();
        let name = if let Some(name) = refs.next() {
            name?
        } else {
            return Ok(None);
        };
        Ok(Some(repo.find_reference(name)?))
    }
}

pub fn get_commit<'a>(repo: &'a Repository, id: &str) -> Result<Option<git2::Commit<'a>>> {
    if let Ok(oid) = git2::Oid::from_str(id) {
        Ok(Some(catch_git!(repo.find_commit(oid), git2::ErrorCode::NotFound, None)))
    } else {
        let reference = match get_ref(&repo, id)? {
            Some(r) => r,
            _ => return Ok(None),
        };
        Ok(Some(reference.peel_to_commit()?))
    }
}
