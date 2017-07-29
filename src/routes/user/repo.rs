use {Context, Error, db, git};
use templates::*;
use types::*;
use super::{not_found, util};

use hayaku::{self, Request, Response, ResponseDone, Status};

// GET /repo/new
pub fn new(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    check_login!(&req.get_cookies(), res, ctx);

    let body = include_str!("../../../templates/user/repo_new.html");
    let tmpl = Template::new(ctx, Some("Create a New Repository"), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /repo/new
pub fn new_post(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let pool = &ctx.db_pool;
    let user_id = try_res!(res, db::read::user_id(pool, username));
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

// GET /{user}/{repo}/settings
pub fn settings(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let params = hayaku::get_path_params(req);
    let user = &params["user"];
    let reponame = &params["repo"];

    // TODO: we probably want a different check here
    if username != user {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "You must own a repo to delete it"));
    }


    let pool = &ctx.db_pool;
    let repo = if let Some(repo) = try_res!(res, db::read::repo(pool, username, reponame)) {
        repo
    } else {
        return not_found(req, res, ctx);
    };

    let body = RepoSettingsTmpl { name: &ctx.name, username: username, repo: repo };
    let tmpl = Template::new(ctx, Some(username), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /{user}/{repo}/settings/name
pub fn settings_name(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let params = hayaku::get_path_params(req);
    let user = &params["user"];
    let reponame = &params["repo"];

    // TODO: we probably want a different check here
    if username != user {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "You must own a repo to delete it"));
    }

    let pool = &ctx.db_pool;
    if !try_res!(res, db::read::repo_exists(pool, username, reponame)) {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "Repo does not exist"));
    }

    let new_name = if let Some(name) = req.form_value("name") {
        name
    } else {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "Invalid data"));
    };

    try_res!(res, db::update::repo_name(pool, username, reponame, &new_name));
    Ok(res.redirect(Status::Found,
                    &format!("/{}/{}", username, new_name),
                    "Repo name changed"))
}

// POST /{user}/{repo}/delete
pub fn delete(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let cookies = req.get_cookies();
    let username = check_login!(&cookies, res, ctx);

    let params = hayaku::get_path_params(req);
    let user = &params["user"];
    let reponame = &params["repo"];

    // TODO: we probably want a different check here
    if username != user {
        return Ok(res.redirect(Status::BadRequest, &format!("/{}/{}", user, reponame),
                               "You must own a repo to delete it"));
    }

    if let Some(name) = req.form_value("delete") {
        if &name != reponame {
            return Ok(res.redirect(Status::BadRequest,
                                   &format!("/{}/{}/settings", user, reponame),
                                   "Incorrect name entered"));
        }
    } else {
        return Ok(res.redirect(Status::BadRequest,
                               &format!("/{}/{}/settings", user, reponame),
                               "You must enter the name of this repo to delete it"));
    }

    let pool = &ctx.db_pool;
    try_res!(res, db::delete::repo(pool, username, reponame));
    try_res!(res, git::delete(ctx, username, reponame));
    Ok(res.redirect(Status::Found, &format!("/{}", username), "Repo deleted"))
}
