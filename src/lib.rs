pub mod compare;
pub mod error;
pub mod probe;
pub mod judge;
pub mod remote_judge;
mod server;

#[derive(Debug)]
pub struct JudgeResult {
    pub status: JudgeStatus,
    pub time: u64,
    pub memory: u64,
    pub stdout:String,
    pub stderr:String,
}

#[derive(Debug)]
pub enum JudgeStatus {
    Uncertain,
    Accept,
    WrongAnswer,
    PatternError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    ComplierLimitExceeded,
    RuntimeError,
}
