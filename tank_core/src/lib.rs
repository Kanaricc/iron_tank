pub mod compare;
pub mod error;
pub mod probe;
pub mod judge;
pub mod remote_judge;
pub mod config;

#[derive(Debug)]
pub struct JudgeResult {
    pub status: JudgeStatus,
    pub time: Option<u64>,
    pub memory: Option<u64>,
    pub stdin:Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug)]
pub enum JudgeStatus {
    Uncertain,
    Accept,
    WrongAnswer,
    PresentationError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    InteractionTimeLimitExceeded,
    ComplierError,
    ComplierLimitExceeded,
    RuntimeError,
}
