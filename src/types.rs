use db::{public_keys, repos, users};

use base64;
use bcrypt::{self, DEFAULT_COST};
use hayaku::Request;
use sha2::{Digest, Sha256};

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
        let username = try_opt!(req.form_value("username"));
        let email = try_opt!(req.form_value("email"));
        let password = try_opt!(req.form_value("password"));
        let password_confirm = try_opt!(req.form_value("password_confirm"));

        if username.is_empty() || email.is_empty() || password.is_empty() ||
            password != password_confirm
        {
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
        let username = try_opt!(req.form_value("username"));
        let password = try_opt!(req.form_value("password"));

        if username.is_empty() || password.is_empty() {
            return None;
        }

        Some(Login {
            username: username,
            password: password,
        })
    }
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

#[derive(Queryable)]
pub struct SshKey {
    pub id: i32,
    pub owner: i32,
    pub fingerprint: String,
    pub content: String,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "public_keys"]
pub struct NewSshKey {
    pub owner: i32,
    pub fingerprint: String,
    pub content: String,
    pub name: String,
}

impl NewSshKey {
    pub fn new(req: &mut Request, owner: i32) -> Option<Self> {
        let name = try_opt!(req.form_value("name"));
        let ssh_key = try_opt!(req.form_value("ssh_key"));
        let fingerprint = try_opt!(NewSshKey::fingerprint(&ssh_key));

        Some(NewSshKey {
            owner: owner,
            fingerprint: fingerprint,
            content: ssh_key,
            name: name,
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
