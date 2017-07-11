use {Error, Result};
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

pub fn user_id(pool: &Pool, username: &str) -> Result<Option<i32>> {
    let conn = pool.get()?;
    // TODO: look into find for unique items
    match users::table.filter(users::username.eq(username))
        .select(users::id)
        .first(&*conn)
    {
        Ok(id) => Ok(Some(id)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn user_exists(pool: &Pool, username: &str) -> Result<bool> {
    Ok(user_id(pool, username)?.is_some())
}

pub fn repo_id(pool: &Pool, username: &str, reponame: &str) -> Result<Option<i64>> {
    let owner = if let Some(owner) = user_id(pool, username)? {
        owner
    } else {
        return Ok(None)
    };

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
    let owner = if let Some(owner) = user_id(pool, username)? {
        owner
    } else {
        return Ok(false);
    };

    let conn = pool.get()?;
    match repos::table.filter(repos::owner.eq(owner))
        .filter(repos::name.eq(reponame))
        .select(repos::private)
        .first(&*conn)
    {
        Ok(private) => Ok(private),
        Err(diesel::result::Error::NotFound) => Ok(false),
        Err(e) => Err(Error::from(e)),
    }
}
pub fn user(pool: &Pool, username: &str) -> Result<Option<User>> {
    let owner = if let Some(owner) = user_id(pool, username)? {
        owner
    } else {
        return Ok(None);
    };

    let conn = pool.get()?;
    let repos = repos::table.filter(repos::owner.eq(owner))
        .select((repos::name, repos::description, repos::owner, repos::private))
        .load::<Repo>(&*conn)?;

    Ok(Some(User {
        username: username.to_string(),
        repos: repos,
    }))

}

pub fn repo(pool: &Pool, username: &str, reponame: &str) -> Result<Option<Repo>> {
    let owner = if let Some(owner) = user_id(pool, username)? {
        owner
    } else {
        return Ok(None);
    };

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

pub fn settings(pool: &Pool, username: &str) -> Result<UserSettings> {
    let owner = user_id(pool, username)?.unwrap();

    let conn = pool.get()?;
    let email = users::table.find(owner)
        .select(users::email)
        .get_result(&*conn)?;

    let keys = public_keys::table.filter(public_keys::owner.eq(owner))
        .load::<SshKey>(&*conn)?;

    Ok(UserSettings {
        username: username.to_string(),
        email: email,
        keys: keys,
    })
}
