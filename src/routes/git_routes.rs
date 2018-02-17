use {db, git};
use super::not_found;

use hayaku::{header, Status};

// GET /{user}/{repo}/info/refs
route!{pull_handshake, req, res, ctx, {
    let username = req.get_param("user");
    let repo = req.get_param("repo");

    let pool = &ctx.db_pool;
    // Make sure that repo exists
    let repo_name = repo.trim_right_matches(".git");
    if !db::read::repo_exists(pool, &username, repo_name)? ||
       db::read::repo_is_private(pool, &username, repo_name)?
    {
        return not_found(req, res, ctx);
    }

    let mode = if let Some(verb) = req.form_value("service") {
        if let Some(mode) = git::AccessMode::new(&verb) {
            mode
        } else {
            res.status(Status::FORBIDDEN);
            return Ok(res.body("Unknown git command"));
        }
    } else {
        res.status(Status::FORBIDDEN);
        let body = format!("Please upgrade your git client. {} does not support git over dumb-http.", ctx.name);
        return Ok(res.body(body));
    };

    if mode == git::AccessMode::Write {
        res.status(Status::FORBIDDEN);
        let body = format!("{} does not support git-receive-pack over HTTP.", ctx.name);
        return Ok(res.body(body));
    }

    let packet = "# service=git-upload-pack\n";
    let length = packet.len() + 4;
    let prefix = format!("{:04x}{}0000", length, packet);

    let mut pack = git::network::info(ctx, &username, &repo)?;
    res.add_header(header::EXPIRES, hval!("Fri, 01 Jan 1980 00:00:00 GMT"));
    res.add_header(header::PRAGMA, hval!("no-cache"));
    res.add_header(header::CACHE_CONTROL, hval!("no-cache, max-age=0, must-revalidate"));
    res.add_header(header::CONTENT_TYPE, hval!("application/x-git-upload-pack-advertisement"));

    // Build body
    let mut body = Vec::new();
    body.append(&mut prefix.into_bytes());
    body.append(&mut pack);
    Ok(res.body(body))
}}

// POST /{user}/{repo}/git-upload-pack
route!{pull, req, res, ctx, {
    let username = req.get_param("user");
    let repo = req.get_param("repo");

    let pack = git::network::pull(ctx, &username, &repo, req.body())?;
    res.add_header(header::EXPIRES, hval!("Fri, 01 Jan 1980 00:00:00 GMT"));
    res.add_header(header::PRAGMA, hval!("no-cache"));
    res.add_header(header::CACHE_CONTROL, hval!("no-cache, max-age=0, must-revalidate"));
    res.add_header(header::CONTENT_TYPE, hval!("application/x-git-upload-pack-result"));
    Ok(res.body(pack))
}}
