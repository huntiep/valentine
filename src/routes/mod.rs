pub mod types;
pub mod user;
mod util;

use {Context, db};
use error::*;

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        user::home(req, res, ctx)
    } else {
        Ok(res.body(include_str!("../../templates/home.html")))
    }
}

// GET /{user}
pub fn user(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        return user::user(req, res, ctx);
    }
    let params = hayaku::get_path_params(req);
    let username = &params["user"];

    let pool = &ctx.db_pool;
    let user =  if let Some(u) = try_res!(res, db::read::user(pool, username)) {
        u
    } else {
        return not_found(req, res, ctx);
    };
    Ok(res.body(""))
}

// GET /{user}/{repo}
pub fn repo(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        user::repo(req, res, ctx)
    } else {
        Ok(res.body(""))
    }
}

pub fn not_found(_req: &mut Request, mut res: Response, _ctx: &Context)
    -> ResponseDone<Error>
{
    res.status(Status::NotFound);
    Ok(res.body(include_str!("../../templates/404.html")))
}

pub fn internal_error(_req: &mut Request, mut res: Response, _ctx: &Context, err: &Error)
    -> ResDone
{
    res.status(Status::InternalServerError);

    match *err {
        _ => {
            return res.body(include_str!("../../templates/internal_error.html"));
        }
    }
    //res.body(include_str!("../../templates/template_error.html"))
}
