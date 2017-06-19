use {Config, Context, db};
use routes::*;

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use hayaku::{Http, Router};
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use toml;

use std::{env, fs, path};
use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn run(config: &str) {
    info!("Starting up");

    info!("Reading config");
    let mut buf = String::new();
    let mut file = fs::File::open(config).expect("Unable to open config file");
    file.read_to_string(&mut buf).expect("Unable to read config file");
    let config: Config = toml::from_str(&buf).expect("Invalid config file");

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

    // Create repository folder
    {
        let path = &config.repo_dir;
        if !path.exists() {
            fs::create_dir(path).unwrap();
        } else if !path.is_dir() {
            panic!("unable to create repository folder, file already exists!");
        }
    }

    let ctx = Context {
        db_pool: pool,
        logins: Arc::new(Mutex::new(HashSet::new())),
        name: String::from("Valentine"),
        repo_dir: config.repo_dir,
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
    router.get("/{user}/{repo}/delete", Arc::new(user::delete_repo));

    let addr = "127.0.0.1:3000".parse().unwrap();
    info!("running server at {}", addr);
    Http::new(router, ctx).listen_and_serve(addr);
}
