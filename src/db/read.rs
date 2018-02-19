use {Context, Error, Result};
use templates::*;
use types::*;
use super::{Pool, public_keys, repos, users};

use diesel;
use diesel::prelude::*;

pub fn check_login(pool: &Pool, login: &Login) -> Result<bool> {
    let conn = pool.get()?;
    let password: String = users::table.filter(users::username.eq(&login.username))
        .select(users::password)
        .first(&*conn)?;
    Ok(::bcrypt::verify(&login.password, &password)?)
}

pub fn user_id(pool: &Pool, username: &str) -> Result<i32> {
    let conn = pool.get()?;
    // TODO: look into find for unique items
    Ok(users::table.filter(users::username.eq(username))
        .select(users::id)
        .first(&*conn)?)
}

pub fn user_exists(pool: &Pool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    Ok(users::table.filter(users::username.eq(username))
        .select(users::id)
        .first::<i32>(&*conn)
        .is_ok())
}

pub fn repo_id(pool: &Pool, username: &str, reponame: &str) -> Result<Option<i64>> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    match repos::table.filter(repos::owner.eq(owner))
        .filter(repos::name.eq(reponame))
        .select(repos::id)
        .first(&*conn)
    {
        Ok(id) => Ok(Some(id)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn repo_exists(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    Ok(repo_id(pool, username, reponame)?.is_some())
}

pub fn repo_is_private(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    match repos::table.filter(repos::owner.eq(owner))
        .filter(repos::name.eq(reponame))
        .select(repos::private)
        .first(&*conn)
    {
        Ok(private) => Ok(private),
        Err(diesel::result::Error::NotFound) => Ok(true),
        Err(e) => Err(Error::from(e)),
    }
}
pub fn user<'a, 'b>(pool: &Pool, username: &'b str, ctx: &'a Context, auth: bool)
    -> Result<Option<User<'a, 'b>>>
{
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let repos = if !auth {
        repos::table.filter(repos::owner.eq(owner))
            .filter(repos::private.eq(false))
            .select((repos::name, repos::description, repos::owner, repos::private))
            .load::<Repo>(&*conn)?
    } else {
        repos::table.filter(repos::owner.eq(owner))
            .select((repos::name, repos::description, repos::owner, repos::private))
            .load::<Repo>(&*conn)?
    };

    Ok(Some(User {
        mount: &ctx.mount,
        username: username,
        repos: repos,
    }))

}

pub fn repo(pool: &Pool, username: &str, reponame: &str) -> Result<Option<Repo>> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let repo = repos::table.filter(repos::owner.eq(owner))
        .filter(repos::name.eq(reponame))
        .select((repos::name, repos::description, repos::owner, repos::private))
        .first::<Repo>(&*conn);

    match repo {
        Ok(repo) => Ok(Some(repo)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn settings<'a, 'b>(pool: &Pool, username: &'b str, ctx: &'a Context)
    -> Result<UserSettings<'a, 'b>>
{
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let email = users::table.find(owner)
        .select(users::email)
        .get_result(&*conn)?;

    let keys = public_keys::table.filter(public_keys::owner.eq(owner))
        .load::<SshKey>(&*conn)?;

    Ok(UserSettings {
        mount: &ctx.mount,
        username: username,
        email: email,
        keys: keys,
        //auth: true,
    })
}

pub fn user_by_key_id(pool: &Pool, id: i32) -> Result<Option<i32>> {
    let conn = pool.get()?;

    match public_keys::table.find(id)
        .select(public_keys::owner)
        .get_result::<i32>(&*conn)
    {
        Ok(key) => Ok(Some(key)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn user_owns_key(pool: &Pool, username: &str, id: i32) -> Result<bool> {
    let user = user_id(pool, username)?;
    let owner = match user_by_key_id(pool, id)? {
        Some(o) => o,
        _ => return Ok(false),
    };

    Ok(owner == user)
}

pub fn user_owns_repo(pool: &Pool, owner: i32, reponame: &str) -> Result<bool> {
    let conn = pool.get()?;

    match repos::table.filter(repos::owner.eq(owner))
        .filter(repos::name.eq(reponame))
        .select(repos::id)
        .first::<i64>(&*conn)
    {
        Ok(_) => Ok(true),
        Err(diesel::result::Error::NotFound) => Ok(false),
        Err(e) => Err(Error::from(e)),
    }
}
