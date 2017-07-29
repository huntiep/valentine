use {Context, Error, db, git};
use templates::*;
use types::*;
use super::{not_found, user, util};

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /{user}/{repo}/issues
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    let issues = try_res!(res, db::read::issues(&ctx.db_pool, username, reponame));
    if issues.is_none() {
        return not_found(req, res, ctx);
    }
    let issues = issues.unwrap();

    let auth = util::check_login(ctx, &req.get_cookies()).is_some();

    let body = IssuesTmpl {
        name: &ctx.name,
        username: username,
        reponame: reponame,
        issues: issues,
        auth: auth,
    };
    let tmpl = Template::new(ctx, Some(reponame), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /{user}/{repo}/issues
pub fn new(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    let pool = &ctx.db_pool;
    let repo = try_res!(res, db::read::repo_id(pool, username, reponame));
    if repo.is_none() {
        return not_found(req, res, ctx);
    }

    let cookies = req.get_cookies();
    let name = if let Some(name) = util::check_login(ctx, &cookies) {
        // TODO: consider a different check here
        if let Some(official) = req.form_value("official") {
            if official == "on".to_string() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let issue = Issue::new_thread(req, repo.unwrap(), name);
    if issue.is_none() {
        return Ok(res.redirect(Status::BadRequest,
                               &format!("/{}/{}/issues", username, reponame),
                               "Invalid input"));
    }
    let mut issue = issue.unwrap();
    try_res!(res, db::create::issue(pool, &mut issue));

    Ok(res.redirect(Status::Found,
                    &format!("/{}/{}/issues/{}", username, reponame, issue.id),
                    "Issue created"))
}

// GET /{user}/{repo}/issues/delete/{thread}
pub fn delete_thread(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];
    // TODO

    Ok(res.body(""))
}

// GET /{user}/{repo}/issues/{thread}
pub fn view(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];
    let thread = parse_param!(req, res, ctx, params, "thread", i64);

    let thread = try_res!(res, db::read::issue(&ctx.db_pool, username, reponame, thread));
    if thread.is_none() {
        return not_found(req, res, ctx);
    }
    let thread = thread.unwrap();

    let auth = util::check_login(ctx, &req.get_cookies()).is_some();

    let body = IssueTmpl {
        name: &ctx.name,
        username: username,
        reponame: reponame,
        thread: thread,
        auth: auth,
    };
    let tmpl = Template::new(ctx, Some(reponame), None, body);
    Ok(res.fmt_body(tmpl))
}

// POST /{user}/{repo}/issues/{thread}
pub fn reply(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];
    let thread = parse_param!(req, res, ctx, params, "thread", i64);

    let pool = &ctx.db_pool;
    let repo = try_res!(res, db::read::repo_id(pool, username, reponame));
    if repo.is_none() {
        return not_found(req, res, ctx);
    }
    let repo = repo.unwrap();

    if !try_res!(res, db::read::issue_exists(pool, repo, thread)) {
        return not_found(req, res, ctx);
    }

    let cookies = req.get_cookies();
    let name = if let Some(name) = util::check_login(ctx, &cookies) {
        // TODO: consider a different check here
        if let Some(official) = req.form_value("official") {
            if official == "on".to_string() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let issue = Issue::new_reply(req, repo, thread, name);
    if issue.is_none() {
        return Ok(res.redirect(Status::BadRequest,
                               &format!("/{}/{}/issues/{}", username, reponame, thread),
                               "Invalid input"));
    }
    let mut issue = issue.unwrap();
    try_res!(res, db::create::reply(pool, &mut issue));

    Ok(res.redirect(Status::Found,
                    &format!("/{}/{}/issues/{}", username, reponame, thread),
                    "Reply created"))
}

// GET /{user}/{repo}/issues/{thread}/delete/{reply}
pub fn delete_post(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];
    // TODO

    Ok(res.body(""))
}
