use crate::JudgeStatus;

mod gpp;

pub trait Compiler {
    fn compile(&self, src: String) -> CompileResult;
    fn check_environment() -> CompilerEnvironmentStatus;
}

#[derive(Debug)]
pub enum CompilerEnvironmentStatus {
    OK { version: String, path: String },
    Missing,
}

pub enum CompileResult {
    OK,
    LimitExceeded,
    CompileError, // TODO: add info
}

impl From<&CompileResult> for JudgeStatus {
    fn from(v: &CompileResult) -> Self {
        match v {
            CompileResult::OK => JudgeStatus::Uncertain,
            CompileResult::LimitExceeded => JudgeStatus::ComplierLimitExceeded,
            CompileResult::CompileError => JudgeStatus::ComplierError,
        }
    }
}
