use {Context, Error, Result};
use templates::*;
use types::*;
use super::Pool;

pub fn check_login(pool: &Pool, login: &Login) -> Result<bool> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT password FROM users WHERE username = ?1"))?;
    let password: String = match stmt.query_row(params![login.username], |row| row.get(0)) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
        Err(e) => return Err(Error::from(e)),
    };
    Ok(::bcrypt::verify(&login.password, &password)?)
}

pub fn user_id(pool: &Pool, username: &str) -> Result<i32> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id FROM users WHERE username = ?1"))?;
    Ok(stmt.query_row(params![username], |row| row.get(0))?)
}

pub fn user_name(pool: &Pool, id: i32) -> Result<String> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT username FROM users WHERE id = ?1"))?;
    Ok(stmt.query_row(params![id], |row| row.get(0))?)
}

pub fn user_exists(pool: &Pool, username: &str) -> Result<bool> {
    Ok(user_id(pool, username).is_ok())
}

pub fn repo_id(pool: &Pool, username: &str, reponame: &str) -> Result<Option<i64>> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT id FROM repos WHERE owner = ?1 AND reponame = ?2"))?;
    match stmt.query_row(params![owner, reponame], |row| row.get(0)) {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn repo_exists(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    Ok(repo_id(pool, username, reponame)?.is_some())
}

pub fn repo_is_private(pool: &Pool, username: &str, reponame: &str) -> Result<bool> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT private FROM repos WHERE owner = ?1 AND reponame = ?2"))?;
    match stmt.query_row(params![owner, reponame], |row| row.get(0)) {
        Ok(private) => Ok(private),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn user<'a, 'b>(pool: &Pool, username: &'b str, ctx: &'a Context, auth: bool)
    -> Result<Option<UserTmpl<'a, 'b>>>
{
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let mut stmt = if !auth {
        conn.prepare(query!("SELECT name, description, owner, private FROM repos WHERE owner = ?1 AND private = false ORDER BY last_updated DESC"))?
    } else {
        conn.prepare(query!("SELECT name, description, owner, private FROM repos WHERE owner = ?1 ORDER BY last_updated DESC"))?
    };
    let rows = stmt.query_map(params![owner], |row| {
        Ok(Repo {
            name: row.get(0)?,
            description: row.get(1)?,
            owner: row.get(2)?,
            private: row.get(3)?,
        })
    })?;
    let mut repos = Vec::new();
    for r in rows {
        repos.push(r?);
    }

    Ok(Some(UserTmpl {
        mount: &ctx.mount,
        username: username,
        repos: repos,
    }))

}

pub fn users<'a>(pool: &Pool, ctx: &'a Context) -> Result<ExploreTmpl<'a>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT name, owner FROM repos WHERE private = false ORDER BY last_updated DESC"))?;
    let rows = stmt.query_map(params![], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;
    let mut repos = Vec::new();
    for r in rows {
        let (name, owner) = r?;
        let owner = user_name(pool, owner)?;
        repos.push((name, owner));
    }

    Ok(ExploreTmpl {
        mount: &ctx.mount,
        repos: repos,
    })
}

pub fn repo(pool: &Pool, username: &str, reponame: &str) -> Result<Option<Repo>> {
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT name, description, owner, private FROM repos WHERE owner = ?1 AND name = ?2"))?;
    match stmt.query_row(params![owner, reponame], |row|
                         Ok(Repo {
                             name: row.get(0)?,
                             description: row.get(1)?,
                             owner: row.get(2)?,
                             private: row.get(3)?,
                         })) {
        Ok(repo) => Ok(Some(repo)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn settings<'a, 'b>(pool: &Pool, username: &'b str, ctx: &'a Context)
    -> Result<UserSettings<'a, 'b>>
{
    let owner = user_id(pool, username)?;

    let conn = pool.get()?;
    let mut stmt = conn.prepare(query!("SELECT email FROM users WHERE id = ?1"))?;
    let email = stmt.query_row(params![username], |row| row.get(0))?;

    let mut stmt = conn.prepare(query!("SELECT id, owner, name, fingerprint, content FROM public_keys WHERE owner = ?1"))?;
    let rows = stmt.query_map(params![owner], |row| {
        Ok(SshKey {
            id: row.get(0)?,
            owner: row.get(1)?,
            name: row.get(2)?,
            fingerprint: row.get(3)?,
            content: row.get(4)?,
        })
    })?;
    let mut keys = Vec::new();
    for r in rows {
        keys.push(r?);
    }

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

    let mut stmt = conn.prepare(query!("SELECT owner FROM public_keys WHERE id = ?1"))?;
    match stmt.query_row(params![id], |row| row.get(0)) {
        Ok(key) => Ok(Some(key)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
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

    let mut stmt = conn.prepare(query!("SELECT id FROM repos WHERE owner = ?1 AND reponame = ?2"))?;
    match stmt.query_row(params![owner, reponame], |row| row.get::<usize, i32>(0)) {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(e) => Err(Error::from(e)),
    }
}
