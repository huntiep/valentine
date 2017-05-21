pub mod types;
pub mod user;
mod util;

use {Context, db};
use error::*;
use templates::*;

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        user::home(req, res, ctx)
    } else {
        let body = include_str!("../../templates/home.html");
        let tmpl = Template::new(ctx, None, body);
        Ok(res.fmt_body(tmpl))
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
    if let Some(user) = try_res!(res, db::read::user(pool, username)) {
        let tmpl = Template::new(ctx, Some(username), user);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}

// GET /{user}/{repo}
pub fn repo(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        user::view_repo(req, res, ctx)
    } else {
        Ok(res.body(""))
    }
}

pub fn not_found(_req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    res.status(Status::NotFound);
    let body = include_str!("../../templates/404.html");
    let tmpl = Template::new(ctx, Some("404"), body);
    Ok(res.fmt_body(tmpl))
}

pub fn internal_error(_req: &mut Request, mut res: Response, ctx: &Context, err: &Error)
    -> ResDone
{
    res.status(Status::InternalServerError);

    match *err {
        _ => {
            let body = include_str!("../../templates/internal_error.html");
            let tmpl = Template::new(ctx, Some("500"), body);
            return res.fmt_body(tmpl);
        }
    }
    //res.body(include_str!("../../templates/template_error.html"))
}
