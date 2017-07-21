#[macro_use] extern crate bart_derive;
extern crate base64;
extern crate bcrypt;
extern crate chrono;
#[macro_use] extern crate clap;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate git2;
#[macro_use] extern crate hayaku;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
#[macro_use(o, kv)] extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
extern crate time;
extern crate toml;

mod cmd;
mod db;
mod git;
mod routes;
mod templates;
mod types;

use clap::{App, Arg, SubCommand};
use slog::Drain;

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = ::std::result::Result<T, Error>;
quick_error! {
    #[derive(Debug, Clone)]
    pub enum Error {
        Base64(err: &'static str) {
            from(_e: ::base64::DecodeError) -> ("base64 error")
        }
        Bcrypt(err: &'static str) {
            from(_e: ::bcrypt::BcryptError) -> ("bcrypt error")
        }
        Git(err: &'static str) {
            from(_e: ::git2::Error) -> ("git error")
        }
        Io(err: &'static str) {
            from(_e: ::std::io::Error) -> ("io error")
        }
        Diesel(err: &'static str) {
            from(_e: ::diesel::result::Error) -> ("diesel error")
        }
        R2D2(err: &'static str) {
            from(_e: ::r2d2::GetTimeout) -> ("r2d2 error")
        }
    }
}

pub type SessionKey = String;
pub type UserName = String;

#[derive(Clone)]
pub struct Context {
    pub db_pool: db::Pool,
    pub logins: Arc<Mutex<HashMap<SessionKey, UserName>>>,
    pub name: String,
    pub repo_dir: PathBuf,
    pub ssh_dir: PathBuf,
    pub bin_path: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo_dir: PathBuf,
    pub ssh_dir: Option<PathBuf>,
    pub db_url: String,
    pub log_path: Option<PathBuf>,
    pub name: Option<String>,
    pub addr: Option<SocketAddr>,
    // TODO
    pub allow_registration: Option<bool>,
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Specifies where to find the config file")
             .takes_value(true))
        .subcommand(SubCommand::with_name("backup")
                    .about("Create a backup of the database and user repositories")
                    .arg(Arg::with_name("FILE")
                         .help("The file to output the backup to e.g. val.tgz")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("ssh")
                    .about("Command used for ssh. Not intended to be used directly")
                    .arg(Arg::with_name("KEYID")
                         .help("The id of this ssh key")
                         .required(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("web")
                    .about("Run the valentine server"))
        .get_matches();

    // Read the config file
    let config_path = matches.value_of("config").unwrap_or("valentine.toml");
    let mut buf = String::new();
    let mut file = fs::File::open(config_path).expect("Unable to open config file");
    file.read_to_string(&mut buf).expect("Unable to read config file");
    let config: Config = toml::from_str(&buf).expect("Invalid config file");

    let log_path = config.log_path.clone().unwrap_or_else(|| PathBuf::from("val.log"));
    // Start the logger
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("unable to open log file");

    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();


    if let Some(matches) = matches.subcommand_matches("backup") {
        let file = matches.value_of("FILE").unwrap();
        cmd::backup::run(file);
    } else if let Some(matches) = matches.subcommand_matches("ssh") {
        cmd::ssh::run(config, matches);
    } else if let Some(_matches) = matches.subcommand_matches("web") {
        let config_path = PathBuf::from(config_path).canonicalize().unwrap();
        cmd::web::run(config, config_path);
    }
}
