mod gpp;

pub trait Compiler{
    fn compile(&self,src:String)->CompileResult;
    fn check_environment()->CompilerEnvironmentStatus;
}

#[derive(Debug)]
pub enum CompilerEnvironmentStatus {
    OK { version: String, path: String },
    Missing,
}

pub enum CompileResult{
    OK,
    LimitExceeded,
    CompileError, // TODO: add info
}