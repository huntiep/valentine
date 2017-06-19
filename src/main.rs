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
extern crate toml;

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

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
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
                    .about("Command used for ssh")
                    .arg(Arg::with_name("config")
                         .short("c")
                         .long("config")
                         .value_name("FILE")
                         .help("Specifies where to find the config file")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("web")
                    .about("Run the valentine server")
                    .arg(Arg::with_name("config")
                         .short("c")
                         .long("config")
                         .value_name("FILE")
                         .help("Specifies where to find the config file")
                         .takes_value(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("backup") {
        let file = matches.value_of("FILE").unwrap();
        cmd::backup::run(file);
    } else if let Some(matches) = matches.subcommand_matches("serve") {
        let config = matches.value_of("config").unwrap_or("valentine.toml");
        cmd::serve::run(config);
    } else if let Some(matches) = matches.subcommand_matches("web") {
        let config = matches.value_of("config").unwrap_or("valentine.toml");
        cmd::web::run(config);
    }
}
