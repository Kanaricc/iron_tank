use std::string;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,Error)]
pub enum Error {
    #[error("entity `{0}` not found")]
    NotFound(String),
    #[error("failed in IO")]
    IO(#[from] std::io::Error),
    #[error("argument provided is error")]
    Argument(String),
    #[error("checker error")]
    UserProgram(String),
    #[error("data error")]
    Data(String),
    #[error("script read an invalid block")]
    UnexpectedBlockType,
    #[error("script error")]
    Script(#[from] rhai::ParseError),
    #[error("judge error")]
    Judge { judge_name: String, msg: String },
    #[error("bytes is not in UTF8")]
    FromUtf8(#[from] string::FromUtf8Error),
    #[error("network error")]
    Request(#[from] reqwest::Error),
    #[error("environment error")]
    Environment(String),
}