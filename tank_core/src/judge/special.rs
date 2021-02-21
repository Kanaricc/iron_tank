use std::io::{Read, Write};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use super::{get_path_of_tankcell, Judge};
use crate::{JudgeResult, JudgeStatus, compile::CompiledProgram, problem::LimitConfig, error::Error, error::Result, probe::ProcessProbe};

pub struct SpecialJudge {
    program: CompiledProgram,
    input: String,
    limit: LimitConfig,
    checker: String,
}

impl SpecialJudge {
    pub fn new(
        program: CompiledProgram,
        input: String,
        memory_limit: u64,
        time_limit: u64,
        checker: String,
    ) -> Self {
        Self {
            program,
            input,
            limit: LimitConfig {
                memory_limit,
                time_limit,
            },
            checker,
        }
    }
}

impl Judge for SpecialJudge {
    fn judge(self) -> Result<JudgeResult> {
        let path = Path::new(&self.program.path);

        let path = fs::canonicalize(path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut command = Command::new(get_path_of_tankcell())
            .arg(path)
            .arg(format!("-m {}", self.limit.memory_limit))
            .arg(format!("-t {}", self.limit.time_limit))
            .arg("-p minimum")
            .arg("--")
            .args(self.program.args)
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
        cin.write_all(self.input.as_bytes())?;
        cin.flush()?;
        drop(cin);
        drop(command.stdin);

        let cout = command.stdout.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdout",
        ))?;
        let cerr = command.stderr.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stderr",
        ))?;
        let probe_res = probe.watching();

        let mut output = String::new();
        cout.read_to_string(&mut output)?;
        let mut errout = String::new();
        cerr.read_to_string(&mut errout)?;

        // check result
        let mut judge_status = if probe_res.get_time_usage() >= self.limit.time_limit {
            JudgeStatus::TimeLimitExceeded
        } else if probe_res.get_peak_memory() >= self.limit.memory_limit * 1024 {
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

        let temp_dir = tempfile::TempDir::new()?;
        let input_tpath = temp_dir.path().join("input.txt");
        let output_tpath = temp_dir.path().join("output.txt");

        fs::write(&input_tpath, &self.input)?;
        fs::write(&output_tpath, &output)?;

        let checker_fullpath = fs::canonicalize(self.checker).unwrap();
        let checker_fullpath = Path::new(&checker_fullpath);

        let check = Command::new(checker_fullpath)
            .arg(input_tpath.to_str().unwrap())
            .arg(output_tpath.to_str().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let checker_output = check.stdout;
        let checker_output = String::from_utf8(checker_output)?;
        let checker_output: Vec<&str> = checker_output.lines().map(|f| f.trim()).collect();

        if let JudgeStatus::Uncertain = judge_status {
            judge_status = match checker_output[0] {
                "same" => JudgeStatus::Accept,
                "different" => JudgeStatus::WrongAnswer,
                "pattern_different" => JudgeStatus::PresentationError,
                _ => Err(Error::UserProgram("checker gives unknown result".into()))?,
            }
        }

        let judge_result = JudgeResult {
            status: judge_status,
            time: probe_res.get_time_usage().into(),
            memory: probe_res.get_peak_memory().into(),
            stdin: None,
            stdout: output.into(),
            stderr: errout.into(),
        };
        Ok(judge_result)
    }
}
