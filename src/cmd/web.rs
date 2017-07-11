use {Config, Context};
use routes::*;

use diesel;
use hayaku::{Http, Router};
use r2d2;
use r2d2_diesel::ConnectionManager;

use std::fs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn run(config: Config) {
    info!("Starting up");

    /*let repo = git2::Repository::open(".").expect("failed to open repo");
    let head = repo.head().expect("failed to get head");
    let oid = head.target().expect("failed to get oid");
    let commit = repo.find_commit(oid).expect("failed to get commit");
    let tree = commit.tree().expect("failed to get tree");
    for entry in tree.iter() {
        println!("{}", entry.name().unwrap());
    }*/

    // Create db connection pool
    let r2d2_config = r2d2::Config::default();
    let manager = ConnectionManager::<diesel::pg::PgConnection>::new(config.db_url);
    let pool = r2d2::Pool::new(r2d2_config, manager).expect("Failed to create pool");

    {
        // Run migrations
        embed_migrations!("migrations");
        let conn = pool.get().unwrap();
        info!("Running migrations");
        embedded_migrations::run(&*conn).expect("failed to run migrations");
    }

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
        logins: Arc::new(Mutex::new(HashMap::new())),
        name: config.name.unwrap_or_else(|| String::from("Valentine")),
        repo_dir: config.repo_dir,
    };

    let mut router = Router::new();
    router.set_not_found_handler(Arc::new(not_found));
    router.set_internal_error_handler(Arc::new(internal_error));
    router.get("/", Arc::new(home));
    router.get("/{user}", Arc::new(user));
    router.get("/{user}/{repo}", Arc::new(repo));
    // TODO: use regex to assert that `repo` ends with .git
    router.get("/{user}/{repo}/info/refs", Arc::new(pull_handshake));
    router.post("/{user}/{repo}/git-upload-pack", Arc::new(pull));

    // User
    router.get("/signup", Arc::new(user::signup));
    router.post("/signup", Arc::new(user::signup_post));
    router.get("/login", Arc::new(user::login));
    router.post("/login", Arc::new(user::login_post));
    router.get("/logout", Arc::new(user::logout));
    router.get("/settings", Arc::new(user::settings));
    router.post("/settings/add-ssh-key", Arc::new(user::add_ssh_key));
    router.get("/repo/new", Arc::new(user::new_repo));
    router.post("/repo/new", Arc::new(user::new_repo_post));
    //router.get("/{user}/{repo}/settings", Arc::new(user::repo_settings));
    router.get("/{user}/{repo}/delete", Arc::new(user::delete_repo));

    let addr = config.addr.unwrap_or_else(|| "127.0.0.1:3000".parse().unwrap());
    info!("running server at {}", addr);
    Http::new(router, ctx).listen_and_serve(addr);
}
