use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use regex::Regex;
use reqwest::{blocking::multipart::Form, blocking::Client};

use crate::{
    compare::CompareMode,
    error::{Error, Result},
    probe::ProcessProbe,
    JudgeResult, JudgeStatus,
};

pub trait Judge {
    fn judge(self) -> Result<JudgeResult>;
}

pub trait RemoteJudge {
    fn get_name(&self) -> String;
    fn prepare(&mut self) -> Result<()>;
    fn judge(self) -> Result<JudgeResult>;

    fn make_error(&self,msg:&str)->Error{
        Error::Judge{
            judge_name:self.get_name(),
            msg:msg.into(),
        }
    }
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
        cin.flush()?;

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
            judge_status = self.comparation.compare(&self.answer, &output).into();
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

pub struct SpecialJudge {
    exec: String,
    input: String,
    memory_limit: u64,
    time_limit: u64,
    checker: String,
}

impl SpecialJudge {
    pub fn new(
        exec: String,
        input: String,
        memory_limit: u64,
        time_limit: u64,
        checker: String,
    ) -> Self {
        Self {
            exec,
            input,
            memory_limit,
            time_limit,
            checker,
        }
    }
}

impl Judge for SpecialJudge {
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
        cin.write_all(self.input.as_bytes())?;
        cin.flush()?;

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
                "pattern_different" => JudgeStatus::PatternError,
                _ => Err(Error::Checker("checker gives unknown result".into()))?,
            }
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
struct OpentrainsJudge {
    username: String,
    password: String,
    sid: Option<String>,
    contest_id: u32,
    problem_id: u32,
    language_id: u32,
    src: String,

    client: Client,
}

impl OpentrainsJudge {
    pub fn new(
        username: String,
        password: String,
        contest_id: u32,
        problem_id: u32,
        language_id: u32,
        src: String,
    ) -> Self {
        Self {
            username,
            password,
            contest_id,
            problem_id,
            language_id,
            src,
            sid: None,
            client: Client::new(),
        }
    }
}

impl RemoteJudge for OpentrainsJudge {
    
    fn prepare(&mut self) -> Result<()> {
        let form = Form::new()
            .text("login", self.username.clone())
            .text("password", self.password.clone())
            .text("locale_id", "0".to_string())
            .text("submit", "Log in".to_string());

        let res = self
            .client
            .post(&format!(
                "http://opentrains.snarknews.info/~ejudge/team.cgi?contest_id={}",
                self.contest_id
            ))
            .multipart(form)
            .send()?;

        let res = res.text()?;

        let rgx = Regex::new(r#"SID="(\d+)""#).unwrap();

        let res = match rgx.captures(&res) {
            Some(x) => x,
            None => {
                return Err(self.make_error("Failed to login."));
            }
        };
        let sid = res.get(1).unwrap().as_str();
        self.sid = Some(sid.to_string());

        Ok(())
    }

    fn judge(self) -> Result<JudgeResult> {
        if let None = self.sid {
            return Err(self.make_error("No session set. Please login first."));
        }
        let sid = self.sid.unwrap();

        todo!()
    }

    fn get_name(&self) -> String {
        "Opentrains Judge".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::ValueCompare;

    #[test]
    fn normal_accept() -> Result<()> {
        // TODO: input with no backspace at end may cause the program waiting for input forever
        let judge = NormalJudge::new(
            "./test_dep/times2".into(),
            "12\n".into(),
            "24".into(),
            256,
            30,
            Box::new(ValueCompare {}),
        );

        let result = judge.judge()?;

        assert!(matches!(result.status, JudgeStatus::Accept));
        assert!(result.time > 0 && result.time <= 30 * 1000);
        assert!(result.memory > 0 && result.memory <= 256 * 1024);

        Ok(())
    }

    #[test]
    fn normal_wrong_answer() -> Result<()> {
        println!("trying testing correct code");
        let judge = NormalJudge::new(
            "./test_dep/times2".into(),
            "12\n".into(),
            "surprise!".into(),
            256,
            30,
            Box::new(ValueCompare {}),
        );

        let result = judge.judge()?;

        assert!(matches!(result.status, JudgeStatus::WrongAnswer));

        Ok(())
    }

    #[test]
    fn normal_time_limit_exceeded() -> Result<()> {
        let judge = NormalJudge::new(
            "./test_dep/tle".into(),
            "".into(),
            "".into(),
            256,
            1000,
            Box::new(ValueCompare {}),
        );

        let result = judge.judge()?;

        assert!(result.time > 1000);
        debug_assert!(matches!(result.status, JudgeStatus::TimeLimitExceeded));
        Ok(())
    }

    #[test]
    fn normal_memory_limit_exceeded() -> Result<()> {
        let judge = NormalJudge::new(
            "./test_dep/mle".into(),
            "".into(),
            "".into(),
            256,
            1000,
            Box::new(ValueCompare {}),
        );

        let result = judge.judge()?;

        assert!(matches!(result.status, JudgeStatus::MemoryLimitExceeded));
        assert!(result.memory > 256 * 1024);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn normal_invalid_path() {
        let judge = NormalJudge::new(
            "./test_dep/whatever".into(),
            "".into(),
            "".into(),
            256,
            1000,
            Box::new(ValueCompare {}),
        );

        judge.judge().unwrap();
    }

    #[test]
    fn opentrains_judge() -> Result<()> {
        let mut judge =
            OpentrainsJudge::new("username".into(), "password".into(), 1, 1, 1, "src".into());
        judge.prepare()?;

        let _result=judge.judge()?;

        Ok(())
    }
}
