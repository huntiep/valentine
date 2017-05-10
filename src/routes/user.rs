use {Context, db};
use error::*;
use templates::*;
use super::types::*;
use super::{not_found, util};

use chrono::Duration;
use hayaku::{self, Cookie, Request, Response, ResponseDone, Status};
use time;

// GET /signup
pub fn signup(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        Ok(res.redirect(Status::Found, "/", "You already have an account"))
    } else {
        let body = include_str!("../../templates/user/signup.html");
        let tmpl = Template::new(ctx, Some("Signup"), body);
        Ok(res.fmt_body(tmpl))
    }
}

// POST /signup
pub fn signup_post(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::Found, "/", "You already have an account"));
    }

    let new_user = NewUser::new(req);
    if new_user.is_none() {
        return Ok(res.redirect(Status::Found, "/signup", "Signup failed"));
    }
    let new_user = new_user.unwrap();

    let pool = &ctx.db_pool;
    try_res!(res, db::create::user(pool, &new_user));
    util::login(new_user.username, &mut res.cookies(), ctx);
    Ok(res.redirect(Status::Found, "/", "Signup sucessfull"))
}

// GET /login
pub fn login(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        Ok(res.redirect(Status::Found, "/", "You are already logged in"))
    } else {
        let body = include_str!("../../templates/user/login.html");
        let tmpl = Template::new(ctx, Some("Login"), body);
        Ok(res.fmt_body(tmpl))
    }
}

// POST /login
pub fn login_post(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::Found, "/", "You are already logged in"));
    }

    let login = Login::new(req);
    if login.is_none() {
        return Ok(res.redirect(Status::Found, "/login", "Login failed"));
    }
    let login = login.unwrap();

    let pool = &ctx.db_pool;
    let login_check = try_res!(res, db::read::check_login(pool, &login));
    if !login_check {
        return Ok(res.redirect(Status::Found, "/login", "Login failed"));
    }

    util::login(login.username, &mut res.cookies(), ctx);
    Ok(res.redirect(Status::Found, "/", "Login successful"))
}

// GET /logout
pub fn logout(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    if let Some(cookie) = cookies.get("session_key") {
        let cookies = res.cookies();
        ctx.logins.lock().unwrap().remove(cookie.value());
        let del_cookie = Cookie::build("session_key", "")
            .max_age(Duration::seconds(0))
            .expires(time::empty_tm())
            .finish();
        cookies.add(del_cookie);

        let del_cookie = Cookie::build("dotcom_user", "")
            .max_age(Duration::seconds(0))
            .expires(time::empty_tm())
            .finish();
        cookies.add(del_cookie);
    }
    Ok(res.redirect(Status::Found, "/", "Logout successful"))
}

pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = util::retrieve_username(&cookies);
    if username.is_none() {
        return Ok(res.redirect(Status::Found, "/login", "Error"));
    }
    let username = username.unwrap();

    let pool = &ctx.db_pool;
    if let Some(user) = try_res!(res, db::read::user(pool, &username)) {
        let tmpl = Template::new(ctx, Some(&username), user);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}

pub fn user(_req: &mut Request, res: Response, _ctx: &Context)
    -> ResponseDone<Error>
{
    Ok(res.body(""))
}

pub fn repo(_req: &mut Request, res: Response, _ctx: &Context)
    -> ResponseDone<Error>
{
    Ok(res.body(""))
}
