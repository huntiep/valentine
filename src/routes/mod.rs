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
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);
    tmpl!(res, ctx, username, Some(navbar), None, HomeTmpl);
}}

// GET /explore
route!{explore, req, res, ctx, {
    let cookies = &req.get_cookies();
    let username = util::check_login(ctx, cookies);
    let navbar = Navbar::new(ctx, username);
    let pool = &ctx.db_pool;
    let users = db::read::users(pool, &ctx)?;
    tmpl!(res, ctx, username, Some(navbar), None, users);
}}

// GET /{user}
route!{user, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = util::check_login(ctx, &cookies);
    let user = req.get_param("user");

    let auth = if let Some(username) = username { username == user } else { false};
    let pool = &ctx.db_pool;
    if let Some(mut body) = db::read::user(pool, &user, &ctx, auth)? {
        let navbar = Navbar::new(ctx, username);
        tmpl!(res, ctx, Some(&user), Some(navbar), None, body);
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

    match err {
        /*
        Error::Base64(e) => res.fmt_body(format!("Base64 error: {}", e)),
        Error::Bcrypt(e) => res.fmt_body(format!("Bcrypt error: {}", e)),
        Error::Chrono(e) => res.fmt_body(format!("Chrono error: {}", e)),
        Error::Git(e) => res.fmt_body(format!("Git error: {}", e)),
        Error::Io(e) => res.fmt_body(format!("Io error: {}", e)),
        Error::R2D2(e) => res.fmt_body(format!("R2D2 error: {}", e)),
        Error::Session(e) => res.fmt_body(format!("Session error: {}", e)),
        Error::Sqlite(e) => res.fmt_body(format!("Sqlite error: {}", e)),
        */
        _ => {
            let body = include_str!("../../templates/internal_error.html");
            let tmpl = Template::new(ctx, Some("500"), None, None, body);
            res.fmt_body(tmpl);
        }
    }
}
