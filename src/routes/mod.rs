pub mod git_routes;
pub mod user;
mod util;

use {Context, Error, db, git};
use templates::*;

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        user::home(req, res, ctx)
    } else {
        let body = HomeTmpl { name: &ctx.name, username: None };
        let tmpl = Template::new(ctx, None, None, body);
        Ok(res.fmt_body(tmpl))
    }
}

// GET /{user}
pub fn user(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        return user::user(req, res, ctx);
    }
    let params = hayaku::get_path_params(req);
    let username = &params["user"];

    let pool = &ctx.db_pool;
    if let Some(mut user) = try_res!(res, db::read::user(pool, username)) {
        user.name = &ctx.name;
        let tmpl = Template::new(ctx, Some(username), None, user);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}

// GET /{user}/{repo}
pub fn repo(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        user::repo::view(req, res, ctx)
    } else {
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
        // TODO
        let tmpl = Template::new(ctx, Some(username), None, repo_git);
        Ok(res.fmt_body(tmpl))
    }
}

pub fn not_found(_req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    res.status(Status::NotFound);
    let body = include_str!("../../templates/404.html");
    let tmpl = Template::new(ctx, Some("404"), None, body);
    Ok(res.fmt_body(tmpl))
}

pub fn internal_error(_req: &mut Request, mut res: Response, ctx: &Context, err: &Error)
    -> ResDone
{
    res.status(Status::InternalServerError);

    match *err {
        _ => {
            let body = include_str!("../../templates/internal_error.html");
            let tmpl = Template::new(ctx, Some("500"), None, body);
            res.fmt_body(tmpl)
        }
    }
}
