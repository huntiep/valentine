pub mod git_routes;
pub mod repo;
pub mod user;
mod util;

use {Context, Error, db};
use templates::*;

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, &cookies);
    let body = HomeTmpl { name: &ctx.name, username: username };
    let tmpl = Template::new(ctx, username, None, body);
    Ok(res.fmt_body(tmpl))
}

// GET /{user}
pub fn user(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = util::check_login(ctx, &cookies);
    let params = hayaku::get_path_params(req);
    let user = &params["user"];

    let pool = &ctx.db_pool;
    if let Some(mut body) = try_res!(res, db::read::user(pool, user)) {
        body.name = &ctx.name;
        body.auth = username.is_some();
        let tmpl = Template::new(ctx, Some(user), None, body);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
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
