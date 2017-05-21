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
