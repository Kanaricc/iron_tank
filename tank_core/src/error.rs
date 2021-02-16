use std::string;

pub type Result<T>=std::result::Result<T,Error>;

#[derive(Debug)]
pub enum Error{
    IO(std::io::Error),
    Argument(String),
    Checker(String),
    Data(String),
    UnexpectedBlockType,
    Judge{
        judge_name:String,
        msg:String,
    },
    FromUtf8(string::FromUtf8Error),
    Request(reqwest::Error),
    Environment(String),
}

impl From<std::io::Error> for Error{
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<string::FromUtf8Error> for Error{
    fn from(err: string::FromUtf8Error) -> Self {
        Self::FromUtf8(err)
    }
}

impl From<reqwest::Error> for Error{
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}