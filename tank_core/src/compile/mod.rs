use crate::JudgeStatus;

mod gpp;
mod python;

pub trait Compiler {
    fn compile(&self, src: String) -> CompileResult;
    fn check_environment() -> CompilerEnvironmentStatus;
}

#[derive(Debug)]
pub enum CompilerEnvironmentStatus {
    OK { version: String, path: String },
    Missing,
}

pub struct CompiledProgram {
    pub path: String,
    pub args: Vec<String>,
}

impl CompiledProgram{
    pub fn new(path:String)->Self{
        Self{
            path,
            args: Vec::new(),
        }
    }

    pub fn new_with_args(path:String,args:Vec<String>)->Self{
        Self{
            path,
            args,
        }
    }
}

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
