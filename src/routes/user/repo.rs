use {Context, Error, db, git};
use templates::*;
use types::*;
use super::{not_found, util};

use chrono::Duration;
use hayaku::{self, Cookie, Request, Response, ResponseDone, Status};
use time;

// GET /{user}/{repo}
pub fn view(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    let pool = &ctx.db_pool;
    if !try_res!(res, db::read::user_exists(pool, username)) {
        return not_found(req, res, ctx);
    }

    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };
    let repo_git = try_res!(res, git::read(ctx, username, repo));
    let tmpl = Template::new(ctx, Some(username), None, repo_git);
    Ok(res.fmt_body(tmpl))
}

// GET /repo/new
pub fn new(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (false, _) = util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    }

    let body = include_str!("../../../templates/user/repo_new.html");
    let tmpl = Template::new(ctx, Some("Create a New Repository"), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /repo/new
pub fn new_post(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = if let (true, Some(name)) = util::check_login(ctx, &cookies) {
        name
    } else {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    };

    let pool = &ctx.db_pool;
    let user_id = try_res!(res, db::read::user_id(pool, username)).unwrap();
    let repo = if let Some(repo) = Repo::new(req, user_id) {
        repo
    } else {
        return Ok(res.redirect(Status::BadRequest, "/repo/new", "Invalid input"));
    };

    if try_res!(res, db::read::repo_exists(pool, username, &repo.name)) {
        return Ok(res.redirect(Status::BadRequest, "/repo/new", "That repo already exists"));
    }
    try_res!(res, db::create::repo(pool, &repo));

    try_res!(res, git::init(ctx, username, repo.name.clone()));

    Ok(res.redirect(Status::Found, &format!("/{}/{}", username, repo.name), "Repo created"))
}

// GET /{user}/{repo}/settings
pub fn settings(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = if let (true, Some(name)) = util::check_login(ctx, &cookies) {
        name
    } else {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    };

    let params = hayaku::get_path_params(req);
    let user = &params["user"];
    let reponame = &params["repo"];

    if username != user {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "You must own a repo to delete it"));
    }


    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    let body = RepoSettingsTmpl { username: username, repo: repo };
    let tmpl = Template::new(ctx, Some(username), None, body);
    Ok(res.fmt_body(tmpl))
}

// GET /{user}/{repo}/delete
pub fn delete(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = if let (true, Some(name)) = util::check_login(ctx, &cookies) {
        name
    } else {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    };

    let params = hayaku::get_path_params(req);
    let user = &params["user"];
    let reponame = &params["repo"];

    if username != user {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "You must own a repo to delete it"));
    }

    let pool = &ctx.db_pool;
    try_res!(res, db::delete::repo(pool, username, reponame));
    try_res!(res, git::delete(ctx, username, reponame));
    Ok(res.redirect(Status::Found, &format!("/{}", username), "Repo deleted"))
}
