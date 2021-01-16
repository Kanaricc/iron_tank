pub type Result<T>=std::result::Result<T,Error>;

#[derive(Debug)]
pub enum Error{
    IO(std::io::Error),
    Argument(String),
}

impl From<std::io::Error> for Error{
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}