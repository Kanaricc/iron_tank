

pub mod compare;
pub mod error;
pub mod probe;

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
