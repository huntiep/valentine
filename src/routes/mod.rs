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
    let tmpl = Template::new(ctx, username, None, body);
    Ok(res.fmt_body(tmpl))
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
        let tmpl = Template::new(ctx, Some(&user), None, body);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}}

route!{not_found, req, res, ctx, {
    res.status(Status::NOT_FOUND);
    let body = include_str!("../../templates/404.html");
    let tmpl = Template::new(ctx, Some("404"), None, body);
    Ok(res.fmt_body(tmpl))
}}

pub fn internal_error(_req: &mut Request, res: &mut Response, ctx: &Context, err: &Error) {
    res.status(Status::INTERNAL_SERVER_ERROR);

    match *err {
        _ => {
            let body = include_str!("../../templates/internal_error.html");
            let tmpl = Template::new(ctx, Some("500"), None, body);
            res.fmt_body(tmpl)
        }
    }
}
