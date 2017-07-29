use Result;
use db::{issues, public_keys, repos, users};

use base64;
use bcrypt::{self, DEFAULT_COST};
use hayaku::Request;
use sha2::{Digest, Sha256};

macro_rules! form_values {
    ( $req:expr, $( $x:expr ),* ) => {
        {
            ($(
                {
                    let x =  try_opt!($req.form_value(stringify!($x)));
                    if x.is_empty() {
                        return None;
                    }
                    x
                }
            ),*)
        }
    };
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub num_repos: i64,
}

impl NewUser {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (username, email) = form_values!(req, username, email);
        let (password, confirm) = form_values!(req, password, password_confirm);

        if password != confirm {
            return None;
        }

        let password_hash = try_opt!(bcrypt::hash(&password, DEFAULT_COST).ok());
        Some(NewUser {
            username: username,
            email: email,
            password: password_hash,
            num_repos: 0,
        })
    }
}

pub struct Login {
    pub username: String,
    pub password: String,
}

impl Login {
    pub fn new(req: &mut Request) -> Option<Self> {
        let (username, password) = form_values!(req, username, password);

        Some(Login {
            username: username,
            password: password,
        })
    }
}

#[derive(Queryable)]
pub struct RepoFull {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub owner: i32,
    pub private: bool,
    pub issue_id: i64,
}

#[derive(Insertable, Queryable)]
#[table_name = "repos"]
pub struct Repo {
    pub name: String,
    pub description: String,
    pub owner: i32,
    pub private: bool,
}

impl Repo {
    pub fn new(req: &mut Request, owner: i32) -> Option<Self> {
        let name = try_opt!(req.form_value("name"));
        let description = try_opt!(req.form_value("description"));
        let private = req.form_value("private");

        if name.is_empty() {
            None
        } else {
            Some(Repo {
                name: name,
                description: description,
                owner: owner,
                private: private == Some(String::from("on")),
            })
        }
    }
}

pub struct RepoItem {
    pub name: String,
    pub obj_type: ::git2::ObjectType,
}

pub struct Commit {
    pub author: String,
    pub time: String,
    pub message: String,
}

impl Commit {
    pub fn new(mut commit: ::git2::Commit) -> Result<Self> {
        let message = commit.summary().unwrap_or("Invalid commit message").to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("Invalid author name").to_string();
        let author_time = author.when();
        let mut time = ::chrono::format::Parsed::new();
        time.set_timestamp(author_time.seconds())?;
        time.set_offset(author_time.offset_minutes() as i64 * 60)?;
        let time = time.to_datetime()?.to_string();
        Ok(Commit {
            author: author_name,
            time: time,
            message: message
        })
    }
}

#[derive(Explode)]
pub enum RepoSrc {
    File(String),
    Dir { items: Vec<RepoItem>, readme: Option<String> },
    Error,
}

#[derive(Queryable)]
pub struct SshKey {
    pub id: i32,
    pub owner: i32,
    pub name: String,
    pub fingerprint: String,
    pub content: String,
}

#[derive(Insertable)]
#[table_name = "public_keys"]
pub struct NewSshKey {
    pub owner: i32,
    pub name: String,
    pub fingerprint: String,
    pub content: String,
}

impl NewSshKey {
    pub fn new(req: &mut Request, owner: i32) -> Option<Self> {
        let (name, ssh_key) = form_values!(req, name, ssh_key);
        let fingerprint = try_opt!(NewSshKey::fingerprint(&ssh_key));

        Some(NewSshKey {
            owner: owner,
            name: name,
            fingerprint: fingerprint,
            content: ssh_key,
        })
    }

    pub fn fingerprint(key: &str) -> Option<String> {
        if let Some(key) = key.split_whitespace().nth(1) {
            let fingerprint_bytes = Sha256::digest(&try_opt!(base64::decode(key).ok()));
            // TODO: this feels a bit inefficient
            let mut fingerprint = String::new();
            for byte in fingerprint_bytes.as_ref() {
                fingerprint.push_str(&format!("{:x}", byte));
            }
            Some(fingerprint)
        } else {
            None
        }
    }
}

#[derive(Insertable, Queryable)]
#[table_name = "issues"]
pub struct Issue {
    pub repo: i64,
    pub id: i64,
    pub parent: i64,
    pub name: Option<String>,
    pub subject: Option<String>,
    pub content: String,
    pub created: ::chrono::NaiveDateTime,
    pub thread: bool,
}

impl Issue {
    pub fn new_thread(req: &mut Request, repo: i64, name: Option<String>) -> Option<Self> {
        let (subject, content) = form_values!(req, subject, content);

        Some(Issue {
            repo: repo,
            id: 0,
            parent: 0,
            name: name,
            subject: Some(subject),
            content: content,
            created: ::chrono::Utc::now().naive_utc(),
            thread: true,
        })
    }

    pub fn new_reply(req: &mut Request, repo: i64, parent: i64, name: Option<String>) -> Option<Self> {
        let content = form_values!(req, content);
        Some(Issue {
            repo: repo,
            id: 0,
            parent: parent,
            name: name,
            subject: None,
            content: content,
            created: ::chrono::Utc::now().naive_utc(),
            thread: false,
        })
    }
}
