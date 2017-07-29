pub mod issues;

use {Context, Error, db, git};
use templates::*;
use super::{not_found, user, util};

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// Check if private repo `name` can be viewed by this request
macro_rules! repo_private {
    ( $name:ident, $req:ident, $res:ident, $ctx:ident ) => {
        {
            let cookies = $req.get_cookies();
            if let Some(username) = util::check_login($ctx, &cookies) {
                let pool = &$ctx.db_pool;
                let owner = try_res!($res, db::read::user_id(pool, username));
                if !try_res!($res, db::read::user_owns_repo(pool, owner, $name)) {
                    return not_found($req, $res, $ctx);
                }
            } else {
                return not_found($req, $res, $ctx);
            }
        }
    };
}


// GET /{user}/{repo}
pub fn view(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let repo_git = try_res!(res, git::read(ctx, username, repo));
    let tmpl = Template::new(ctx, Some(reponame), None, repo_git);
    Ok(res.fmt_body(tmpl))
}

// GET /{user}/{repo}/tree/{branch}/{filepath}
pub fn src(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];
    let branch = &params["branch"];
    let filepath = &params["filepath"];

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let src = try_res!(res, git::read_src(ctx, username, &repo, branch, filepath));
    if src.is_none() {
        return not_found(req, res, ctx);
    }

    let body = RepoSrcTmpl {
        name: &ctx.name,
        username: username,
        repo: repo,
        // TODO: maybe something else?
        filename: filepath,
        src: src.unwrap(),
    };
    let tmpl = Template::new(ctx, Some(reponame), None, body);
    Ok(res.fmt_body(tmpl))
}

// GET /{user}/{repo}/log
pub fn log(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    if repo.private {
        repo_private!(reponame, req, res, ctx);
    }

    let log = try_res!(res, git::log(ctx, username, reponame));
    let body = RepoLogTmpl {
        name: &ctx.name,
        username: username,
        repo: repo,
        log: log
    };
    let tmpl = Template::new(ctx, Some(reponame), None, body);
    Ok(res.fmt_body(tmpl))
}