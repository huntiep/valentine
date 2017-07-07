use Context;

use chrono::Duration;
use hayaku::{Cookie, CookieJar};
use rand::{OsRng, Rng};

pub fn check_login<'a>(ctx: &Context, cookies: &'a CookieJar) -> (bool, Option<&'a str>) {
    if let Some(cookie) = cookies.get("session_key") {
        if let Some(name) = ctx.logins.lock().unwrap().get(cookie.value()) {
            if let Some(cookie) = cookies.get("dotcom_user") {
                if cookie.value() == name {
                    return (true, Some(cookie.value()));
                }
            }
        }
    }
    (false, None)
}

pub fn login(username: String, cookies: &mut CookieJar, ctx: &Context) {
    let key: String = OsRng::new().unwrap().gen_ascii_chars().take(50).collect();
    ctx.logins.lock().unwrap().insert(key.clone(), username.clone());
    let cookie = Cookie::build("session_key", key)
        .secure(false)
        .http_only(false)
        .path("/")
        .max_age(Duration::days(1))
        .finish();
    cookies.add(cookie);

    let cookie = Cookie::build("dotcom_user", username)
        .secure(false)
        .http_only(false)
        .path("/")
        .max_age(Duration::days(1))
        .finish();
    cookies.add(cookie);
}

pub fn retrieve_username<'a>(cookies: &'a CookieJar) -> Option<&'a str> {
    let cookie = try_opt!(cookies.get("dotcom_user"));
    Some(cookie.value())
}
