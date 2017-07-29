use {Context, Error, db, git};
use templates::*;
use super::{not_found, user, util};

use hayaku::{self, Request, Response, ResDone, ResponseDone, Status};

// GET /{user}/{repo}/issues
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}

// POST /{user}/{repo}/issues/new
pub fn new(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}

// GET /{user}/{repo}/issues/delete/{thread}
pub fn delete_thread(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}

// GET /{user}/{repo}/issues/{thread}
pub fn view(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}

// POST /{user}/{repo}/issues/{thread}/reply
pub fn reply(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}

// GET /{user}/{repo}/issues/{thread}/delete/{reply}
pub fn delete_post(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let reponame = &params["repo"];

    Ok(res.body(""))
}
