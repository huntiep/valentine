use {db, git};
use templates::*;
use super::{not_found, util};

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
    tmpl!(res, ctx, Some(&reponame), None, None, repo_git);
}}

// GET /{user}/{repo}/tree/{name}/{*filepath}
route!{src, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let name = req.get_param("name");
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

    let src = git::read_src(ctx, &username, &repo, &name, &filepath)?;
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
    tmpl!(res, ctx, Some(&reponame), None, None, body);
}}

// GET /{user}/{repo}/commits/{branch}
route!{log, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let branch = req.get_param("branch");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let log = if let Some(log) = git::log(ctx, &username, &reponame, &branch)? {
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
    tmpl!(res, ctx, Some(&reponame), None, None, body);
}}

// GET /{user}/{repo}/commit/{commit}
route!{commit, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let commit = req.get_param("commit");

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, &username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }


    let body = git::commit(ctx, &username, repo, &commit)?;
    if body.is_none() {
        return not_found(req, res, ctx);
    }
    tmpl!(res, ctx, Some(&reponame), None, None, body.unwrap());
}}

// TODO
// GET /{user}/{repo}/blob/{commit}/{*filepath}
route!{blob, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let commit = req.get_param("commit");
    let file = req.get_param("filepath");
    Ok(())
}}

// TODO
// GET /{user}/{repo}/raw/{commit}/{*filepath}
route!{raw, req, res, ctx, {
    let username = req.get_param("user");
    let reponame = req.get_param("repo");
    let commit = req.get_param("commit");
    let file = req.get_param("filepath");
    Ok(())
}}
