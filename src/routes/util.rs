use Context;

use chrono::Duration;
use hayaku::{Cookie, CookieJar, Response};
use rand::{OsRng, Rng};

pub fn check_login(ctx: &Context, cookies: &CookieJar) -> bool {
    if let Some(cookie) = cookies.get("session_key") {
        ctx.logins.lock().unwrap().get(cookie.value()).is_some()
    } else {
        false
    }
}

pub fn login(username: String, res: &mut Response, ctx: &Context) {
    let key: String = OsRng::new().unwrap().gen_ascii_chars().take(50).collect();
    ctx.logins.lock().unwrap().insert(key.clone());
    let cookie = Cookie::build("session_key", key)
        .secure(false)
        .http_only(false)
        .path("/")
        .max_age(Duration::days(1))
        .finish();
    res.set_cookie(cookie);

    let cookie = Cookie::build("dotcom_user", username)
        .secure(false)
        .http_only(false)
        .path("/")
        .max_age(Duration::days(1))
        .finish();
    res.set_cookie(cookie);
}

pub fn retrieve_username<'a>(cookies: &'a CookieJar) -> Option<&'a str> {
    let cookie = try_opt!(cookies.get("dotcom_user"));
    Some(cookie.value())
}
