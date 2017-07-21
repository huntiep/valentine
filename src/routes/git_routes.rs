use {Context, Error, db, git};
use super::not_found;

use hayaku::{self, headers, Request, Response, ResponseDone, Status};

// GET /{user}/{repo}/info/refs
pub fn pull_handshake(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let repo = &params["repo"];

    let pool = &ctx.db_pool;
    // Make sure that repo exists
    let repo_name = repo.trim_right_matches(".git");
    if !try_res!(res, db::read::repo_exists(pool, username, repo_name)) ||
       try_res!(res, db::read::repo_is_private(pool, username, repo_name))
    {
        return not_found(req, res, ctx);
    }

    let mode = if let Some(verb) = req.form_value("service") {
        if let Some(mode) = git::AccessMode::new(&verb) {
            mode
        } else {
            res.status(Status::Forbidden);
            return Ok(res.body("Unknown git command"));
        }
    } else {
        res.status(Status::Forbidden);
        let body = format!("Please upgrade your git client. {} does not support git over dumb-http.", ctx.name);
        return Ok(res.body(body));
    };

    if mode == git::AccessMode::Write {
        res.status(Status::Forbidden);
        let body = format!("{} does not support git-receive-pack over HTTP.", ctx.name);
        return Ok(res.body(body));
    }

    let packet = "# service=git-upload-pack\n";
    let length = packet.len() + 4;
    let prefix = format!("{:04x}{}0000", length, packet);

    let mut pack = try_res!(res, git::network::info(ctx, username, repo));
    // TODO: set cache headers
    //res.add_header(headers::Expires("Fri, 01 Jan 1980 00:00:00 GMT"));
    //res.add_header(headers::Pragma("no-cache"));
    //res.add_header(headers::CacheControl("no-cache, max-age=0, must-revalidate"));
    res.add_header(headers::ContentType("application/x-git-upload-pack-advertisement"));

    // Build body
    let mut body = Vec::new();
    body.append(&mut prefix.into_bytes());
    body.append(&mut pack);
    Ok(res.body(body))
}

// POST /{user}/{repo}/git-upload-pack
pub fn pull(req: &mut Request, mut res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    let params = hayaku::get_path_params(req);
    let username = &params["user"];
    let repo = &params["repo"];

    let pack = try_res!(res, git::network::pull(ctx, username, repo, req.body()));
    // TODO: set cache headers
    res.add_header(headers::ContentType("application/x-git-upload-pack-result"));
    Ok(res.body(pack))
}
