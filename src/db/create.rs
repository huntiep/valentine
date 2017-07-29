use Result;
use types::*;
use super::{Pool, issues, public_keys, repos, users};

use diesel;
use diesel::prelude::*;

pub fn user(pool: &Pool, user: &NewUser) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert(user).into(users::table)
        .execute(&*conn)?;
    Ok(())
}

pub fn public_key(pool: &Pool, key: &NewSshKey) -> Result<SshKey> {
    let conn = pool.get()?;
    Ok(diesel::insert(key).into(public_keys::table)
        .get_result::<SshKey>(&*conn)?)
}

pub fn repo(pool: &Pool, repo: &Repo) -> Result<()> {
    let conn = pool.get()?;
    diesel::insert(repo).into(repos::table).execute(&*conn)?;
    diesel::update(users::table.find(repo.owner))
        .set(users::num_repos.eq(users::num_repos + 1))
        .execute(&*conn)?;
    Ok(())
}

pub fn issue(pool: &Pool, issue: &mut Issue) -> Result<()> {
    let conn = pool.get()?;
    let repo = diesel::update(repos::table.find(issue.repo))
        .set(repos::issue_id.eq(repos::issue_id + 1))
        .get_result::<RepoFull>(&*conn)?;
    issue.id = repo.issue_id;
    issue.parent = repo.issue_id;
    diesel::insert(issue).into(issues::table).execute(&*conn)?;
    Ok(())
}

pub fn reply(pool: &Pool, reply: &mut Issue) -> Result<()> {
    let conn = pool.get()?;
    let repo = diesel::update(repos::table.find(reply.repo))
        .set(repos::issue_id.eq(repos::issue_id + 1))
        .get_result::<RepoFull>(&*conn)?;
    reply.id = repo.issue_id;
    diesel::insert(reply).into(issues::table).execute(&*conn)?;
    Ok(())
}
