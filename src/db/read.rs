use {Error, Result};
use templates::*;
use types::*;
use super::Pool;

pub fn check_login(pool: &Pool, login: &Login) -> Result<bool> {
    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/check_login.sql"),
                          &[&login.username])?;
    if rows.is_empty() {
        Ok(false)
    } else {
        let row = rows.get(0);
        let password_hash: String = row.get(0);
        let valid = ::bcrypt::verify(&login.password, &password_hash)?;
        Ok(valid)
    }
}

pub fn user_uuid(pool: &Pool, username: &str) -> Result<Option<Uuid>> {
    let conn = pool.get()?;

    // Get user uuid
    let rows = conn.query(include_str!("../sql/read/user_uuid.sql"),
                          &[&username])?;
    if rows.is_empty() {
        return Ok(None);
    }
    let row = rows.get(0);
    let owner: Uuid = row.get(0);
    Ok(Some(owner))
}

pub fn user_exists(pool: &Pool, username: &str) -> Result<bool> {
    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/user_exists.sql"),
                          &[&username])?;
    Ok(!rows.is_empty())
}

pub fn repo_exists(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    let owner = if let Some(owner) = user_uuid(pool, username)? {
        owner
    } else {
        return Ok(false);
    };

    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/repo_exists.sql"),
                          &[&owner, &reponame])?;
    Ok(!rows.is_empty())
}

pub fn repo_is_private(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    let owner = if let Some(owner) = user_uuid(pool, username)? {
        owner
    } else {
        return Ok(false);
    };

    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/repo_is_private.sql"),
                          &[&owner, &reponame])?;
    if rows.is_empty() {
        Ok(false)
    } else {
        let row = rows.get(0);
        Ok(row.get(0))
    }
}

pub fn user(pool: &Pool, username: &str) -> Result<Option<User>> {
    let owner = if let Some(owner) = user_uuid(pool, username)? {
        owner
    } else {
        return Ok(None);
    };

    let conn = pool.get()?;
    let rows = conn.query(include_str!("../sql/read/user.sql"),
                          &[&owner])?;

    let mut repos = Vec::new();
    for row in rows.iter() {
        let repo = Repo {
            name: row.get(1),
            description: row.get(2),
            private: row.get(4),
        };
        repos.push(repo);
    }

    Ok(Some(User {
        username: username.to_string(),
        repos: repos,
    }))
}

pub fn settings(pool: &Pool, username: &str) -> Result<UserSettings> {
    let conn = pool.get()?;
    let rows = conn.query("SELECT uuid, username, email FROM users WHERE username = $1;",
                          &[&username])?;
    if rows.is_empty() {
        return Err(Error::Postgres("postgres error"));
    }
    let row = rows.get(0);
    let uuid: Uuid = row.get(0);
    let name: String = row.get(1);
    let email: String = row.get(2);

    let rows = conn.query("SELECT name, fingerprint FROM public_key WHERE owner = $1",
                          &[&uuid])?;

    let mut keys = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        let key = SshKey {
            fingerprint: row.get(1),
            content: String::new(),
            name: row.get(0),
        };
        keys.push(key);
    }

    Ok(UserSettings {
        username: name,
        email: email,
        keys: keys,
    })
}
