use crate::JudgeStatus;

pub mod gpp;
pub mod python;

pub trait CompilerDescriptor {
    fn support_sufix() -> Vec<&'static str>;
    fn check_environment() -> CompilerEnvironmentStatus;
}

pub trait Compiler {
    fn compile(&self, src: String) -> CompileResult;
}

#[derive(Debug)]
pub enum CompilerEnvironmentStatus {
    OK { version: String, path: String },
    Missing,
}
#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub path: String,
    pub args: Vec<String>,
}

impl CompiledProgram {
    pub fn new(path: String) -> Self {
        Self {
            path,
            args: Vec::new(),
        }
    }

    pub fn new_with_args(path: String, args: Vec<String>) -> Self {
        Self { path, args }
    }
}
#[derive(Debug)]
pub enum CompileResult {
    OK(CompiledProgram),
    LimitExceeded,
    CompileError, // TODO: add info
}

impl From<&CompileResult> for JudgeStatus {
    fn from(v: &CompileResult) -> Self {
        match v {
            CompileResult::OK(_) => JudgeStatus::Uncertain,
            CompileResult::LimitExceeded => JudgeStatus::ComplierLimitExceeded,
            CompileResult::CompileError => JudgeStatus::ComplierError,
        }
    }
}
