use std::{fs, io::{Read, Write}, path::Path, process::{Command, Stdio}};

use crate::{JudgeResult, JudgeStatus, compare::CompareMode, error::Result, probe::ProcessProbe};

pub trait Judge {
    fn judge(self) -> Result<JudgeResult>;
}

pub struct NormalJudge {
    exec: String,
    input: String,
    answer: String,
    memory_limit: u64,
    time_limit: u64,
    comparation: Box<dyn CompareMode>,
}

impl NormalJudge {
    pub fn new(
        exec: String,
        input: String,
        answer: String,
        memory_limit: u64,
        time_limit: u64,
        comparation: Box<dyn CompareMode>,
    ) -> Self {
        Self {
            exec,
            input,
            answer,
            time_limit,
            memory_limit,
            comparation,
        }
    }
}

impl Judge for NormalJudge {
    fn judge(self) -> Result<JudgeResult> {
        let path = Path::new(&self.exec);

        let path = fs::canonicalize(path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut command = Command::new("./target/debug/tank_cell")
            .arg(path)
            .arg(format!("-m {}", self.memory_limit))
            .arg(format!("-t {}", self.time_limit))
            .arg("-p minimum")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let pid = command.id();
        let probe = ProcessProbe::new(pid)?;

        let cin = command.stdin.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdin",
        ))?;
        cin.write_all(self.input.into_bytes().as_slice())?;

        let cout = command
            .stdout
            .as_mut().ok_or(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "failed to open stdout",
            ))?;
        let cerr = command
            .stderr
            .as_mut().ok_or(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "failed to open stderr",
            ))?;
        let probe_res = probe.watching();

        let mut output = String::new();
        cout.read_to_string(&mut output)?;
        let mut errout = String::new();
        cerr.read_to_string(&mut errout)?;

        // check result
        let mut judge_status = if probe_res.get_time_usage() >= self.time_limit {
            JudgeStatus::TimeLimitExceeded
        } else if probe_res.get_peak_memory() >= self.memory_limit * 1024 {
            JudgeStatus::MemoryLimitExceeded
        } else if errout.find("bad_alloc").is_some() {
            // fix: struct like vector which does not allocate memory gradually
            // may touch the wall when memory is still below the limit
            // even we give two times more of it.
            JudgeStatus::MemoryLimitExceeded
        } else if probe_res.get_status() != 0 {
            JudgeStatus::RuntimeError
        } else {
            JudgeStatus::Uncertain
        };

        if let JudgeStatus::Uncertain = judge_status {
            judge_status=self.comparation.compare(&self.answer, &output).into();
        }

        let judge_result = JudgeResult {
            status: judge_status,
            time: probe_res.get_time_usage(),
            memory: probe_res.get_peak_memory(),
            stdout: output,
            stderr: errout,
        };

        Ok(judge_result)
    }
}
