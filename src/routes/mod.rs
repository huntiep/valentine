pub mod types;
pub mod user;
mod util;

use Context;
use error::*;

use hayaku::{Request, Response, ResDone, ResponseDone, Status};

// GET /
pub fn home(req: &mut Request, res: Response, ctx: &Context)
    -> ResponseDone<Error>
{
    if util::check_login(ctx, &req.get_cookies()) {
        user::home(req, res, ctx)
    } else {
        Ok(res.body(include_str!("../../templates/home.html")))
    }
}

pub fn not_found(_req: &mut Request, mut res: Response, _ctx: &Context)
    -> ResponseDone<Error>
{
    res.status(Status::NotFound);
    Ok(res.body(include_str!("../../templates/404.html")))
}

pub fn internal_error(_req: &mut Request, mut res: Response, _ctx: &Context, err: &Error)
    -> ResDone
{
    res.status(Status::InternalServerError);

    match *err {
        _ => {
            return res.body(include_str!("../../templates/internal_error.html"));
        }
    }
    //res.body(include_str!("../../templates/template_error.html"))
}
