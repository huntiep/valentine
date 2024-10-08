use {db, git};
use templates::*;
use types::RepoSrc;
use super::{not_found, util};

use hayaku::header;

macro_rules! read_repo {
    ( $username:ident, $reponame:ident, $req:ident, $res:ident, $ctx:ident ) => {
        {
            let pool = &$ctx.db_pool;
            let repo = if let Some(repo) = db::read::repo(pool, &$username, &$reponame)? {
                repo
            } else {
                return not_found($req, $res, $ctx);
            };

            // Check if private repo can be viewed by this request
            if repo.private {
                let cookies = $req.get_cookies();
                // Private repos can only be viewed by logged in users
                if let Some(username) = util::check_login($ctx, &cookies) {
                    let owner = db::read::user_id(pool, username)?;
                    if !db::read::user_owns_repo(pool, owner, &$reponame)? {
                        return not_found($req, $res, $ctx);
                    }
                } else {
                    return not_found($req, $res, $ctx);
                }
            }
            repo
        }
    };
}

// GET /{user}/{repo}
route!{view, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");

    let repo = read_repo!(username, reponame, req, res, ctx);
    let repo_git = git::read(ctx, &username, repo)?;

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, repo_git);
}}

// GET /{user}/{repo}/log
route!{log_default, req, res, ctx, {
    let user = req.get_param("user");
    let repo = req.get_param("repo");
    redirect!(res, ctx, format!("{}/{}/log/HEAD", user, repo), "Viewing log from HEAD");
}}

// GET /{user}/{repo}/log/{id}
route!{log, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let mut id = req.get_param("id");

    if let Some(s) = req.form_value("s") {
        id = s;
    }
    let repo = read_repo!(username, reponame, req, res, ctx);
    let log = if let Some(log) = git::log(ctx, &username, &reponame, &id)? {
        log
    } else {
        return not_found(req, res, ctx);
    };

    let body = RepoLogTmpl {
        mount: &ctx.mount,
        username: &username,
        repo: repo,
        id: req.get_param("id"),
        log: log.0,
        commits: log.1,
        next: log.2,
    };

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body);
}}

// GET /{user}/{repo}/refs
route!{refs_list, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");

    let repo = read_repo!(username, reponame, req, res, ctx);
    let body = git::refs(ctx, &username, repo)?;

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body);
}}

// GET /{user}/{repo}/refs/{id}
route!{commit, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let id = req.get_param("id");

    let repo = read_repo!(username, reponame, req, res, ctx);
    let body = git::commit(ctx, &username, repo, &id)?;
    if body.is_none() {
        return not_found(req, res, ctx);
    }

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body.unwrap());
}}

// GET /{user}/{repo}/refs/{id}/{*filepath}
route!{src, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let id = req.get_param("id");
    let mut filepath = req.get_param("filepath");
    if filepath.ends_with('/') { filepath.pop(); }

    let repo = read_repo!(username, reponame, req, res, ctx);
    let src = git::read_src(ctx, &username, &repo, &id, &filepath)?;
    if src.is_none() {
        return not_found(req, res, ctx);
    }

    let p: Vec<_> = filepath.split('/').collect();
    let mut path = Vec::new();
    let mut url = String::new();
    for part in p {
        url.push('/');
        url.push_str(part);
        path.push((url.clone(), part.to_string()));
    }

    let body = RepoSrcTmpl {
        mount: &ctx.mount,
        username: &username,
        repo: repo,
        url: format!("{}/{}/refs/{}", username, reponame, id),
        path: path,
        // TODO: maybe something else?
        filename: &filepath,
        src: src.unwrap(),
    };

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body);
}}

// GET /{user}/{repo}/refs/{id}/raw/{*filepath}
route!{raw, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let id = req.get_param("id");
    let filepath = req.get_param("filepath");

    let repo = read_repo!(username, reponame, req, res, ctx);
    // TODO: maybe just do read_file here
    let src = match git::read_src(ctx, &username, &repo, &id, &filepath)? {
        Some(s) => s,
        None => return not_found(req, res, ctx),
    };

    match src {
        RepoSrc::Dir { .. } => {
            redirect!(res, ctx,
                      format!("{}/{}/refs/{}/{}", username, reponame, id, filepath),
                      "Can't view raw directories");
        }
        RepoSrc::Error => return not_found(req, res, ctx),
        RepoSrc::File{ file, size } => {
            res.add_header(header::CONTENT_TYPE, hval!("text/plain; charset=utf-8"));
            let body = file.iter().map(|(_, s)| s.as_str()).collect::<String>();
            let body = file.iter().map(|(_, s)| s.as_str())
                .fold(String::new(), |mut acc, s| {
                    acc.push_str(s);
                    acc.push('\n');
                    acc
                });
            ok!(res.body(body));
        }
    }
}}
