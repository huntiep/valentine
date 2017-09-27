use {db, git};
use templates::*;
use types::*;
use super::{not_found, util};

use hayaku::Status;

// GET /repo/new
route!{new, req, res, ctx, {
    check_login!(&req.get_cookies(), res, ctx);

    let body = include_str!("../../../templates/user/repo_new.html");
    tmpl!(res, ctx, Some("Create a New Repository"), None, body);
}}

// POST /repo/new
route!{new_post, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let user_id = db::read::user_id(pool, username)?;
    let repo = if let Some(repo) = Repo::new(req, user_id) {
        repo
    } else {
        redirect!(res, ctx, "repo/new", "Invalid input");
    };

    if db::read::repo_exists(pool, username, &repo.name)? {
        redirect!(res, ctx, "repo/new", "That repo already exists");
    }
    db::create::repo(pool, &repo)?;

    git::init(ctx, username, &repo.name)?;

    redirect!(res, ctx, format!("{}/{}", username, repo.name), "Repo created");
}}

// GET /{user}/{repo}/settings
route!{settings, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let user = req.get_param("user");
    let reponame = req.get_param("repo");

    // TODO: we probably want a different check here
    if username != user {
        redirect!(res, ctx, format!("{}/{}", user, reponame), "You must own a repo to delete it");
    }


    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = db::read::repo(pool, username, &reponame)? {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    let body = RepoSettingsTmpl { name: &ctx.name, username: username, repo: repo };
    tmpl!(res, ctx, Some(username), None, body);
}}

// POST /{user}/{repo}/settings/name
route!{settings_name, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let user = req.get_param("user");
    let reponame = req.get_param("repo");

    // TODO: we probably want a different check here
    if username != user {
        redirect!(res, ctx, format!("{}/{}", user, reponame), "You must own a repo to delete it");
    }

    let pool = &ctx.db_pool;
    if !db::read::repo_exists(pool, username, &reponame)? {
        redirect!(res, ctx, format!("{}/{}", user, reponame), "Repo does not exist");
    }

    let new_name = if let Some(name) = req.form_value("name") {
        name
    } else {
        redirect!(res, ctx, format!("{}/{}", user, reponame), "Invalid  data");
    };

    db::update::repo_name(pool, username, &reponame, &new_name)?;
    redirect!(res, ctx, format!("{}/{}", username, new_name), "Repo name changed");
}}

// POST /{user}/{repo}/delete
route!{delete, req, res, ctx, {
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let user = req.get_param("user");
    let reponame = req.get_param("repo");

    // TODO: we probably want a different check here
    if username != user {
        redirect!(res, ctx, format!("{}/{}", user, reponame), "You must own a repo to delete it");
    }

    if let Some(name) = req.form_value("delete") {
        if name != reponame {
        redirect!(res, ctx, format!("{}/{}/settings", user, reponame), "Incorrect name entered");
        }
    } else {
        redirect!(res, ctx, format!("{}/{}/settings", user, reponame),
                  "You must enter the name of this repo to delete it");
    }

    let pool = &ctx.db_pool;
    db::delete::repo(pool, username, &reponame)?;
    git::delete(ctx, username, &reponame)?;
    redirect!(res, ctx, format!("{}", username), "Repo deleted");
}}
