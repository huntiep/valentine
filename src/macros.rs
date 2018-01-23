macro_rules! ok {
    ( $expr:expr ) => {
        $expr;
        return Ok(());
    };
}

macro_rules! route {
    ( $name:ident, $req:ident, $res:ident, $ctx:ident, $body:expr) => {
        #[allow(unused_mut, unused_variables)]
        pub fn $name($req: &mut ::hayaku::Request, $res: &mut ::hayaku::Response, $ctx: &::Context)
            -> ::Result<()>
        {
            $body
        }
    };
}

macro_rules! redirect {
    ( $res:ident, $ctx:ident, $path:expr, $msg:expr) => {
        ok!($res.redirect(Status::FOUND, &format!("{}{}", $ctx.mount, $path), $msg));
    };
}

macro_rules! check_login {
    ( $cookies:expr, $res:expr, $ctx:expr ) => {
        {
            if let Some(name) = util::check_login($ctx, $cookies) {
                name
            } else {
                return Ok($res.redirect(Status::FORBIDDEN, "/login", "You must be logged in for this"));
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

macro_rules! tmpl {
    ( $res:ident, $ctx:ident, $name:expr, $navbar:expr, $err:expr, $body:expr ) => {
        ok!($res.fmt_body(::templates::Template::new($ctx, $name, $navbar, $err, $body)));
    };
}

macro_rules! hval {
    ($val:expr) => {
        ::hayaku::header::HeaderValue::from_static($val)
    };
}

macro_rules! catch_git {
    ($res:expr, $err:pat, $ret:expr) => {
        match $res {
            Ok(t) => t,
            Err(e) => match e.code() {
                $err => return Ok($ret),
                _ => return Err(::Error::from(e))
            },
        }
    };
}
