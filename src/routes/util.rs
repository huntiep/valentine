use {Context, Result};

use chrono::Duration;
use hayaku::{Cookie, CookieJar};

pub fn check_login<'a>(ctx: &Context, cookies: &'a CookieJar) -> Result<Option<&'a str>> {
    if let Some(cookie) = cookies.get("session_key") {
        if let Some(session) = ctx.logins.lock().unwrap().read(cookie.value())? {
            let name: String = session.metadata()?;
            if let Some(cookie) = cookies.get("dotcom_user") {
                if cookie.value() == name {
                    return Ok(Some(cookie.value()));
                }
            }
        }
    }
    Ok(None)
}

pub fn login(username: String, cookies: &mut CookieJar, ctx: &Context) -> Result<()> {
    let key = ctx.logins.lock().unwrap().generate(Duration::days(30), username.clone())?;

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
    Ok(())
}
