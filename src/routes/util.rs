use Context;

use chrono::Duration;
use hayaku::{Cookie, CookieJar};

pub fn check_login<'a>(ctx: &Context, cookies: &'a CookieJar) -> Option<&'a str> {
    if let Some(cookie) = cookies.get("session_key") {
        if let Some(session) = ctx.logins.lock().unwrap().read(cookie.value()) {
            let name: String = session.metadata().unwrap();
            if let Some(cookie) = cookies.get("dotcom_user") {
                if cookie.value() == name {
                    return Some(cookie.value());
                }
            }
        }
    }
    None
}

pub fn login(username: String, cookies: &mut CookieJar, ctx: &Context) {
    let key = ctx.logins.lock().unwrap().generate(Duration::days(30), username.clone());

    let cookie = Cookie::build("session_key", key)
        .secure(true)
        .http_only(true)
        .path("/")
        .max_age(time::Duration::days(30))
        .finish();
    cookies.add(cookie);

    let cookie = Cookie::build("dotcom_user", username)
        .secure(true)
        .http_only(true)
        .path("/")
        .max_age(time::Duration::days(30))
        .finish();
    cookies.add(cookie);
}

pub fn logout(req_cookies: &CookieJar, res_cookies: &mut CookieJar, ctx: &Context) {
    if let Some(cookie) = req_cookies.get("session_key") {
        ctx.logins.lock().unwrap().remove(cookie.value());
        let del_cookie = Cookie::build("session_key", "")
            .max_age(time::Duration::seconds(0))
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .finish();
        res_cookies.add(del_cookie);

        let del_cookie = Cookie::build("dotcom_user", "")
            .max_age(time::Duration::seconds(0))
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .finish();
        res_cookies.add(del_cookie);
    }
}
