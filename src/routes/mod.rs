pub mod git_routes;
pub mod repo;
pub mod user;
mod util;

use {Context, Error, db};
use templates::*;

use hayaku::{Request, Response, Status};

// GET /
route!{home, req, res, ctx, {
    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, &cookies);
    let body = HomeTmpl { name: &ctx.name, username: username };
    tmpl!(res, ctx, username, None, None, body);
}}

// GET /{user}
route!{user, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = util::check_login(ctx, &cookies);
    let user = req.get_param("user");

    let pool = &ctx.db_pool;
    if let Some(mut body) = db::read::user(pool, &user)? {
        body.name = &ctx.name;
        body.auth = username.is_some();
        tmpl!(res, ctx, Some(&user), None, None, body);
    } else {
        not_found(req, res, ctx)
    }
}}

route!{not_found, req, res, ctx, {
    res.status(Status::NOT_FOUND);
    let body = include_str!("../../templates/404.html");
    tmpl!(res, ctx, Some("404"), None, None, body);
}}

pub fn internal_error(_req: &mut Request, res: &mut Response, ctx: &Context, err: &Error) {
    res.status(Status::INTERNAL_SERVER_ERROR);

    match *err {
        _ => {
            let body = include_str!("../../templates/internal_error.html");
            let tmpl = Template::new(ctx, Some("500"), None, None, body);
            res.fmt_body(tmpl);
        }
    }
}
