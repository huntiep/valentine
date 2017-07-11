use {Context, Error, db, git};
use templates::*;
use types::*;
use super::{not_found, util};

use chrono::Duration;
use hayaku::{self, Cookie, Request, Response, ResponseDone, Status};
use time;

// GET /signup
pub fn signup(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        Ok(res.redirect(Status::BadRequest, "/", "You already have an account"))
    } else {
        let body = include_str!("../../templates/user/signup.html");
        let tmpl = Template::new(ctx, Some("Signup"), None, body);
        Ok(res.fmt_body(tmpl))
    }
}

// POST /signup
pub fn signup_post(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::BadRequest, "/", "You already have an account"));
    }

    let new_user = NewUser::new(req);
    if new_user.is_none() {
        return Ok(res.redirect(Status::BadRequest, "/signup", "Signup failed"));
    }
    let new_user = new_user.unwrap();

    let pool = &ctx.db_pool;
    try_res!(res, db::create::user(pool, &new_user));
    try_res!(res, git::create_user(ctx, &new_user.username));
    util::login(new_user.username, &mut res.cookies(), ctx);
    Ok(res.redirect(Status::Found, "/", "Signup sucessfull"))
}

// GET /login
pub fn login(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        Ok(res.redirect(Status::BadRequest, "/", "You are already logged in"))
    } else {
        let body = include_str!("../../templates/user/login.html");
        let tmpl = Template::new(ctx, Some("Login"), None, body);
        Ok(res.fmt_body(tmpl))
    }
}

// POST /login
pub fn login_post(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (true, _) = util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::BadRequest, "/", "You are already logged in"));
    }

    let login = Login::new(req);
    if login.is_none() {
        return Ok(res.redirect(Status::BadRequest, "/login", "Login failed"));
    }
    let login = login.unwrap();

    let pool = &ctx.db_pool;
    let login_check = try_res!(res, db::read::check_login(pool, &login));
    if !login_check {
        return Ok(res.redirect(Status::BadRequest, "/login", "Login failed"));
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

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = util::retrieve_username(&cookies);
    if username.is_none() {
        return Ok(res.redirect(Status::Forbidden, "/login", "Error"));
    }
    let username = username.unwrap();
    info!("read cookie");

    let pool = &ctx.db_pool;
    if let Some(user) = try_res!(res, db::read::user(pool, username)) {
        let tmpl = Template::new(ctx, Some(username), None, user);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}

pub fn user(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];

    let pool = &ctx.db_pool;
    if let Some(user) = try_res!(res, db::read::user(pool, username)) {
        let tmpl = Template::new(ctx, Some(username), None, user);
        Ok(res.fmt_body(tmpl))
    } else {
        not_found(req, res, ctx)
    }
}

pub fn view_repo(_req: &mut Request, res: Response, _ctx: &Context)
    -> ResponseDone<Error>
{
    Ok(res.body(""))
}

// GET /settings
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

    let settings = try_res!(res, db::read::settings(&ctx.db_pool, username));
    let tmpl = Template::new(ctx, Some("Settings"), None, settings);
    Ok(res.fmt_body(tmpl))
}

// POST /settings/add-ssh-key
pub fn add_ssh_key(req: &mut Request, res: Response, ctx: &Context)
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
    let ssh_key = if let Some(key) = NewSshKey::new(req, user_id) {
        key
    } else {
        return Ok(res.redirect(Status::Forbidden, "/settings", "Invalid data"));
    };
    // TODO validate key
    /*if thrussh::parse_public_key_base64(ssh_key).is_err() {
        // TODO
        let settings = "";
        let msg = "Invalid ssh key!";
        let tmpl = Template::new(ctx, Some("Settings"), Some(msg), settings);
        res.status(Status::BadRequest);
        return Ok(res.fmt_body(tmpl));
    }*/
    let key = try_res!(res, db::create::public_key(pool, &ssh_key));
    try_res!(res, git::add_ssh_key(ctx, &key));

    Ok(res.redirect(Status::Ok, "/settings", "SSH key added"))
}

// TODO
pub fn delete_ssh_key(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    Ok(res.body(""))
}

// GET /repo/new
pub fn new_repo(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if let (false, _) = util::check_login(ctx, &req.get_cookies()) {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    }

    let body = include_str!("../../templates/user/repo_new.html");
    let tmpl = Template::new(ctx, Some("Create a New Repository"), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /repo/new
pub fn new_repo_post(req: &mut Request, res: Response, ctx: &Context)
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

// GET /{user}/{repo}/delete
pub fn delete_repo(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let name = if let (true, Some(name)) = util::check_login(ctx, &cookies) {
        name
    } else {
        return Ok(res.redirect(Status::Forbidden, "/login",
                               "You must be logged in for this operation"));
    };

    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let repo_name = &params["repo"];

    if name != username {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", username, repo_name),
                               "You must own a repo to delete it"));
    }

    let pool = &ctx.db_pool;
    try_res!(res, db::delete::repo(pool, username, repo_name));
    try_res!(res, git::delete(ctx, username, repo_name));
    Ok(res.redirect(Status::Found, &format!("/{}", username), "Repo deleted"))
}
