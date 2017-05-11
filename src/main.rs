#[macro_use] extern crate bart_derive;
extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate git2;
#[macro_use] extern crate hayaku;
#[macro_use(info, log)] extern crate log;
extern crate postgres;
#[macro_use] extern crate quick_error;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rand;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate time;

mod db;
mod error;
mod routes;
mod templates;

use routes::*;

use dotenv::dotenv;
use hayaku::{Http, Router};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Context {
    pub db_pool: db::Pool,
    pub logins: Arc<Mutex<HashSet<String>>>,
    pub name: String,
}

fn main() {
    dotenv().ok();
    env_logger::init().unwrap();
    info!("Starting up");

    /*let repo = git2::Repository::open(".").expect("failed to open repo");
    let head = repo.head().expect("failed to get head");
    let oid = head.target().expect("failed to get oid");
    let commit = repo.find_commit(oid).expect("failed to get commit");
    let tree = commit.tree().expect("failed to get tree");
    for entry in tree.iter() {
        println!("{}", entry.name().unwrap());
    }*/

    // Read database url from `.env`
    let db_url = env::var("DATABASE_URL").expect("$DATABASE_URL must be set");
    info!("db url: {}", db_url);

    // Create db connection pool
    let r2d2_config = r2d2::Config::default();
    let manager = PostgresConnectionManager::new(db_url, TlsMode::None).unwrap();
    let pool = r2d2::Pool::new(r2d2_config, manager).expect("Failed to create pool");

    // Create the tables if they do not already exist
    info!("Creating tables");
    db::create::tables(&pool).expect("failed to create tables");

    let ctx = Context {
        db_pool: pool,
        logins: Arc::new(Mutex::new(HashSet::new())),
        name: String::from("wanker"),
    };

    let mut router = Router::new();
    router.set_not_found_handler(Arc::new(not_found));
    router.set_internal_error_handler(Arc::new(internal_error));
    router.get("/", Arc::new(home));
    router.get("/{user}", Arc::new(user));
    router.get("/{user}/{repo}", Arc::new(repo));

    // User
    router.get("/signup", Arc::new(user::signup));
    router.post("/signup", Arc::new(user::signup_post));
    router.get("/login", Arc::new(user::login));
    router.post("/login", Arc::new(user::login_post));
    router.get("/logout", Arc::new(user::logout));
    router.get("/repo/new", Arc::new(user::new_repo));
    router.post("/repo/new", Arc::new(user::new_repo_post));

    let addr = "127.0.0.1:3000".parse().unwrap();
    Http::new(router, ctx).listen_and_serve(addr);
}
