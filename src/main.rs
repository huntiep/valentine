#[macro_use] extern crate bart_derive;
extern crate bcrypt;
extern crate chrono;
#[macro_use] extern crate clap;
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

mod cmd;
mod db;
mod repo;
mod routes;
mod templates;

use routes::*;

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = ::std::result::Result<T, Error>;
quick_error! {
    #[derive(Debug, Clone)]
    pub enum Error {
        Bcrypt(err: &'static str) {
            from(_e: ::bcrypt::BcryptError) -> ("bcrypt error")
        }
        Git(err: &'static str) {
            from(_e: ::git2::Error) -> ("git error")
        }
        Io(err: &'static str) {
            from(_e: ::std::io::Error) -> ("io error")
        }
        PostGres(err: &'static str) {
            from(_e: ::postgres::error::Error) -> ("postgres error")
        }
        R2D2(err: &'static str) {
            from(_e: ::r2d2::GetTimeout) -> ("r2d2 error")
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub db_pool: db::Pool,
    pub logins: Arc<Mutex<HashSet<String>>>,
    pub name: String,
    pub repo_dir: PathBuf,
}

fn main() {
    dotenv().ok();
    env_logger::init().unwrap();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("backup")
                    .about("Create a backup of the database and user repositories")
                    .arg(Arg::with_name("FILE")
                         .help("The file to output the backup to e.g. val.tgz")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("serve")
                    .about("Command used for ssh"))
        .subcommand(SubCommand::with_name("web")
                    .about("Run the valentine server"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("backup") {
        let file = matches.value_of("FILE").unwrap();
        cmd::backup::run(file);
    } else if let Some(_matches) = matches.subcommand_matches("serve") {
        cmd::serve::run();
    } else if let Some(_matches) = matches.subcommand_matches("web") {
        cmd::web::run();
    }
}

/*fn main() {
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

    // Create repository folder
    let path = path::Path::new("valentine-repos");
    if !path.exists() {
        fs::create_dir(path).unwrap();
    } else if !path.is_dir() {
        panic!("unable to create repository folder, file already exists!");
    }

    let ctx = Context {
        db_pool: pool,
        logins: Arc::new(Mutex::new(HashSet::new())),
        name: String::from("Valentine"),
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
}*/
