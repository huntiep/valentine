macro_rules! check_login {
    ( $cookies:expr, $res:expr, $ctx:expr ) => {
        {
            if let Some(name) = util::check_login($ctx, $cookies) {
                name
            } else {
                return Ok($res.redirect(Status::Forbidden, "/login", "You must be logged in for this"));
            }
        }
    };
}

macro_rules! parse_param {
    ( $req:ident, $res:ident, $ctx:ident, $params:ident, $name:expr, $t:ty) => {
        {
            match $params[$name].parse::<$t>() {
                Ok(p) => p,
                Err(_) => return super::not_found($req, $res, $ctx),
            }
        }
    };
}
