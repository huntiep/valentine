use {db, git};
use templates::*;
use types::RepoSrc;
use super::{not_found, util};

use hayaku::header;

// Check if private repo `name` can be viewed by this request
macro_rules! repo_private {
    ( $name:ident, $req:ident, $res:ident, $ctx:ident ) => {
        {
            let cookies = $req.get_cookies();
            if let Some(username) = util::check_login($ctx, &cookies) {
                let pool = &$ctx.db_pool;
                let owner = db::read::user_id(pool, username)?;
                if !db::read::user_owns_repo(pool, owner, &$name)? {
                    return not_found($req, $res, $ctx);
                }
            } else {
                return not_found($req, $res, $ctx);
            }
        }
    };
}

// GET /{user}/{repo}
route!{view, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let repo_git = git::read(ctx, &username, repo)?;

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, repo_git);
}}

// GET /{user}/{repo}/refs/{id}/{*filepath}
route!{src, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let id = req.get_param("id");
    let filepath = req.get_param("filepath");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let src = git::read_src(ctx, &username, &repo, &id, &filepath)?;
    if src.is_none() {
        return not_found(req, res, ctx);
    }

    let body = RepoSrcTmpl {
        name: &ctx.name,
        username: &username,
        repo: repo,
        // TODO: maybe something else?
        filename: &filepath,
        src: src.unwrap(),
    };

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body);
}}

// GET /{user}/{repo}/log
route!{log_default, req, res, ctx, {
    let user = req.get_param("user");
    let repo = req.get_param("repo");
    redirect!(res, ctx, format!("{}/{}/log/HEAD", user, repo), "Viewing log from HEAD");
}}

// GET /{user}/{repo}/log/{name}
route!{log, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let name = req.get_param("name");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let log = if let Some(log) = git::log(ctx, &username, &reponame, &name)? {
        log
    } else {
        return not_found(req, res, ctx);
    };

    let body = RepoLogTmpl {
        name: &ctx.name,
        username: &username,
        repo: repo,
        log: log
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

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

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

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let body = git::commit(ctx, &username, repo, &id)?;
    if body.is_none() {
        return not_found(req, res, ctx);
    }

    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);

    tmpl!(res, ctx, Some(&reponame), Some(navbar), None, body.unwrap());
}}

// GET /{user}/{repo}/refs/{id}/raw/{*filepath}
route!{raw, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let id = req.get_param("id");
    let filepath = req.get_param("filepath");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let src = match git::read_src(ctx, &username, &repo, &id, &filepath)? {
        Some(s) => s,
        None => return not_found(req, res, ctx),
    };

    match src {
        RepoSrc::Dir { .. } => {
            res.add_header(header::CONTENT_TYPE, hval!("text/plain; charset=utf-8"));
            redirect!(res, ctx,
                      format!("{}/{}/refs/{}/{}", username, reponame, id, filepath),
                      "Can't view raw directories");
        }
        RepoSrc::Error => return not_found(req, res, ctx),
        RepoSrc::File(f) => { ok!(res.body(f)); }
    }
}}
