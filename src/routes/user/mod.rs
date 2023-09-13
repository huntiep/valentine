pub mod repo;

use {db, git};
use templates::*;
use types::*;
use super::{not_found, util};

use hayaku::Status;

// GET /signup
route!{signup, req, res, ctx, {
    if !ctx.signup {
        not_found(req, res, ctx)
    } else if util::check_login(ctx, &req.get_cookies()).is_some() {
        redirect!(res, ctx, "", "You already have an account");
    } else {
        let navbar = Navbar::new(ctx, None);
        let body = include_str!("../../../templates/user/signup.html");
        tmpl!(res, ctx, Some("Signup"), Some(navbar), None, body);
    }
}}

// POST /signup
route!{signup_post, req, res, ctx, {
    if !ctx.signup {
        return not_found(req, res, ctx);
    } else if util::check_login(ctx, &req.get_cookies()).is_some() {
        redirect!(res, ctx, "", "You already have an account");
    }

    let new_user = NewUser::new(req);
    if new_user.is_none() {
        redirect!(res, ctx, "signup", "Signup failed");
    }
    let new_user = new_user.unwrap();

    let pool = &ctx.db_pool;
    db::create::user(pool, &new_user)?;
    git::create_user(ctx, &new_user.username)?;
    util::login(new_user.username, &mut res.cookies(), ctx);
    redirect!(res, ctx, "", "Signup successful");
}}

// GET /login
route!{login, req, res, ctx, {
    if util::check_login(ctx, &req.get_cookies()).is_some() {
        redirect!(res, ctx, "", "You are already logged in");
    } else {
        let navbar = Navbar::new(ctx, None);
        let body = include_str!("../../../templates/user/login.html");
        tmpl!(res, ctx, Some("Login"), Some(navbar), None, body);
    }
}}

// POST /login
route!{login_post, req, res, ctx, {
    if util::check_login(ctx, &req.get_cookies()).is_some() {
        redirect!(res, ctx, "", "You are already logged in");
    }

    let login = Login::new(req);
    if login.is_none() {
        redirect!(res, ctx, "login", "Login failed");
    }
    let login = login.unwrap();

    let pool = &ctx.db_pool;
    let login_check = db::read::check_login(pool, &login)?;
    if !login_check {
        redirect!(res, ctx, "login", "Login failed");
    }

    util::login(login.username, &mut res.cookies(), ctx);
    redirect!(res, ctx, "", "Login successful");
}}

// GET /logout
route!{logout, req, res, ctx, {
    /*
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
    */
    util::logout(&req.get_cookies(), &mut res.cookies(), ctx);
    redirect!(res, ctx, "", "Logout successful");
}}

// GET /settings
route!{settings, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let mut settings = db::read::settings(&ctx.db_pool, username, ctx)?;
    let navbar = Navbar::new(ctx, Some(username));
    tmpl!(res, ctx, Some("Settings"), Some(navbar), None, settings);
}}

// POST /settings/add-ssh-key
route!{add_ssh_key, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let user_id = db::read::user_id(pool, username)?;
    let ssh_key = if let Some(key) = NewSshKey::new(req, user_id) {
        key
    } else {
        redirect!(res, ctx, "settings", "Invalid data");
    };
    // TODO validate key
    let key = db::create::public_key(pool, &ssh_key)?;
    git::add_ssh_key(ctx, &key)?;

    redirect!(res, ctx, "settings", "SSH key added");
}}

// GET /settings/delete-ssh-key/{id}
route!{delete_ssh_key, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);
    let id = parse_param!(req, res, ctx, "id", i32);

    let pool = &ctx.db_pool;
    if db::read::user_owns_key(pool, username, id)? {
        git::delete_ssh_key(ctx, id)?;
        db::delete::public_key(pool, id)?;
        redirect!(res, ctx, "settings", "Key deleted");
    } else {
        redirect!(res, ctx, "settings", "Key does not exist");
    }
}}
