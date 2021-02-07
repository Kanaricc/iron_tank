use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

use libc::{kill, SIGKILL};

use crate::{
    compare::ComparisionMode,
    config::{ComparisionModeConfig, LimitConfig},
    error::{Error, Result},
    probe::ProcessProbe,
    JudgeResult, JudgeStatus,
};

pub trait Judge {
    fn judge(self) -> Result<JudgeResult>;
}

pub struct NormalJudge {
    exec: String,
    input: String,
    answer: String,
    limit: LimitConfig,
    comparation: Box<dyn ComparisionMode>,
}

impl NormalJudge {
    pub fn new(
        exec: String,
        input: String,
        answer: String,
        memory_limit: u64,
        time_limit: u64,
        comparation: Box<dyn ComparisionMode>,
    ) -> Self {
        Self {
            exec,
            input,
            answer,
            limit: LimitConfig {
                time_limit: time_limit,
                memory_limit: memory_limit,
            },
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

        let mut command = Command::new(get_path_of_tankcell())
            .arg(path)
            .arg(format!("-m {}", self.limit.memory_limit))
            .arg(format!("-t {}", self.limit.time_limit))
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

        if let JudgeStatus::Uncertain = judge_status {
            judge_status = self.comparation.compare(&self.answer, &output).into();
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

pub struct SpecialJudge {
    exec: String,
    input: String,
    limit: LimitConfig,
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
        let path = Path::new(&self.exec);

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
            println!("{:?}",probe_res);
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
                _ => Err(Error::Checker("checker gives unknown result".into()))?,
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

struct InteractiveJudge {
    exec: String,
    input: String,
    limit: LimitConfig,
    interactor: String,
}

enum InteractiveMessage {
    UserOut(Vec<u8>),
    InteractorOut(Vec<u8>),
    UserQuit,
    InteractorQuit,
}

impl InteractiveJudge {
    pub fn new(exec: String, input: String, limit: LimitConfig, interactor: String) -> Self {
        Self {
            exec,
            input,
            limit,
            interactor,
        }
    }
}

impl Judge for InteractiveJudge {
    fn judge(self) -> Result<JudgeResult> {
        let interactor_fullpath = fs::canonicalize(self.interactor).unwrap();
        let interactor_fullpath = Path::new(&interactor_fullpath);

        let interactor = Command::new(interactor_fullpath)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let interactor_pid = interactor.id();
        let mut iin = interactor.stdin.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdin for interactor",
        ))?;
        let mut iout = interactor.stdout.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdout for interactor",
        ))?;
        let mut ierr = interactor.stderr.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stderr for interactor",
        ))?;

        let path = Path::new(&self.exec);
        let path = fs::canonicalize(path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let command = Command::new(get_path_of_tankcell())
            .arg(path)
            .arg(format!("-m {}", self.limit.memory_limit))
            .arg(format!("-t {}", self.limit.time_limit))
            .arg("-p minimum")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let pid = command.id();
        let probe = ProcessProbe::new(pid)?;
        let mut cin = command.stdin.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdin",
        ))?;
        let mut cout = command.stdout.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stdout",
        ))?;
        let mut cerr = command.stderr.ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "failed to open stderr",
        ))?;

        let (sender, receiver) = channel();

        let csender = sender.clone();
        let isender = sender.clone();
        let _user_thread = thread::spawn(move || {
            let mut buf: [u8; 1024] = [0; 1024];
            loop {
                let res = cout.read(&mut buf);
                match res {
                    Ok(len) => {
                        // TODO: len=0
                        if len == 0 {
                            continue;
                        }
                        let mut vec = Vec::from(buf);
                        vec.truncate(len);
                        csender.send(InteractiveMessage::UserOut(vec)).unwrap();
                    }
                    Err(_err) => {
                        println!("user reading error: {:?}", _err);
                        // TODO:
                        csender.send(InteractiveMessage::UserQuit).unwrap();
                    }
                }
            }
        });
        let _interactor_thread = thread::spawn(move || {
            let mut buf: [u8; 1024] = [0; 1024];
            loop {
                match iout.read(&mut buf) {
                    Ok(len) => {
                        // TODO: len=0
                        if len == 0 {
                            continue;
                        }
                        let mut vec = Vec::from(buf);
                        vec.truncate(len);
                        isender
                            .send(InteractiveMessage::InteractorOut(vec))
                            .unwrap();
                    }
                    Err(err) => {
                        println!("interactor reading error: {:?}", err);
                        // TODO:
                        isender.send(InteractiveMessage::InteractorQuit).unwrap();
                    }
                }
            }
        });

        let input = Arc::new(Mutex::new(Vec::new()));
        let output = Arc::new(Mutex::new(Vec::new()));

        let b_input = input.clone();
        let b_output = output.clone();
        let broker = thread::spawn(move || {
            loop {
                match receiver.recv().unwrap() {
                    InteractiveMessage::UserOut(buf) => {
                        b_output
                            .lock()
                            .unwrap()
                            .push(std::string::String::from_utf8(buf.clone()).unwrap());
                        if let Err(_err) = iin.write_all(buf.as_slice()) {
                            println!("interactor writing error: {:?}", _err);
                            // TODO: handle err
                            break;
                        }
                    }
                    InteractiveMessage::InteractorOut(buf) => {
                        b_input
                            .lock()
                            .unwrap()
                            .push(std::string::String::from_utf8(buf.clone()).unwrap());
                        if let Err(_err) = cin.write_all(buf.as_slice()) {
                            println!("user writing error: {:?}", _err);
                            // TODO: handle err
                            break;
                        }
                    }
                    InteractiveMessage::UserQuit => {
                        // TODO:
                        break;
                    }
                    InteractiveMessage::InteractorQuit => {
                        // TODO:
                        break;
                    }
                }
            }
        });

        // wait for user quitting
        let probe_res = probe.watching();
        // interactor MUST quit before user, or it will be killed.
        sender.send(InteractiveMessage::UserQuit).unwrap();
        unsafe {
            kill(interactor_pid as i32, SIGKILL);
        }
        // wait for broker
        broker.join().unwrap();

        // join output and input
        let output = output.lock().unwrap().join("\n");
        let input = input.lock().unwrap().join("\n");

        let mut user_errout = String::new();
        cerr.read_to_string(&mut user_errout)?;
        let mut interactor_errout = String::new();
        ierr.read_to_string(&mut interactor_errout)?;

        // check result
        let mut judge_status = if probe_res.get_time_usage() >= self.limit.time_limit {
            JudgeStatus::TimeLimitExceeded
        } else if probe_res.get_peak_memory() >= self.limit.memory_limit * 1024 {
            JudgeStatus::MemoryLimitExceeded
        } else if user_errout.find("bad_alloc").is_some() {
            // fix: struct like vector which does not allocate memory gradually
            // may touch the wall when memory is still below the limit
            // even we give two times more of it.
            JudgeStatus::MemoryLimitExceeded
        } else if probe_res.get_status() != 0 {
            println!("{:?}", probe_res);
            JudgeStatus::RuntimeError
        } else {
            JudgeStatus::Uncertain
        };

        let interactor_errout: Vec<&str> = interactor_errout.lines().map(|f| f.trim()).collect();

        if let JudgeStatus::Uncertain = judge_status {
            if interactor_errout.len() <= 0 {
                return Err(Error::Checker("interactor gives no response".into()));
            }
            judge_status = match interactor_errout[0] {
                "same" => JudgeStatus::Accept,
                "different" => JudgeStatus::WrongAnswer,
                "pattern_different" => JudgeStatus::PresentationError,
                _ => Err(Error::Checker("interactor gives unknown result".into()))?,
            }
        }

        let judge_result = JudgeResult {
            status: judge_status,
            time: probe_res.get_time_usage().into(),
            memory: probe_res.get_peak_memory().into(),
            stdin: input.into(),
            stdout: output.into(),
            stderr: user_errout.into(),
        };
        Ok(judge_result)
    }
}

pub fn launch_normal_case_judge(
    exec: &str,
    input_file: &str,
    answer_file: &str,
    limit: &LimitConfig,
    comparision_mode: &ComparisionModeConfig,
) -> Result<JudgeResult> {
    let path = Path::new(exec);
    let input_file_path = Path::new(input_file);
    let answer_file_path = Path::new(answer_file);

    if !path.exists() || !input_file_path.exists() || !answer_file_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "code, input or answer not found",
        )
        .into());
    }

    let input = fs::read_to_string(input_file_path)?;
    let answer = fs::read_to_string(answer_file_path)?;

    let comparation: Box<dyn ComparisionMode> = comparision_mode.into();

    let judge = NormalJudge::new(
        exec.into(),
        input,
        answer,
        limit.memory_limit,
        limit.time_limit,
        comparation,
    );
    let judge_result = judge.judge()?;

    Ok(judge_result)
}

pub fn launch_special_case_judge(
    exec: &str,
    input_file: &str,
    checker: &str,
    limit: &LimitConfig,
) -> Result<JudgeResult> {
    let path = Path::new(exec);
    let input_file_path = Path::new(input_file);
    let checker_path = Path::new(checker);

    if !path.exists() || !input_file_path.exists() || !checker_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "code, input or checker not found",
        )
        .into());
    }

    let input = fs::read_to_string(input_file_path)?;

    let judge = SpecialJudge::new(
        exec.into(),
        input,
        limit.memory_limit,
        limit.time_limit,
        checker.into(),
    );
    let judge_result = judge.judge()?;

    Ok(judge_result)
}

pub fn launch_interactive_case_judge(
    exec: &str,
    input_file: &str,
    interactor: &str,
    limit: &LimitConfig,
) -> Result<JudgeResult> {
    let path = Path::new(exec);
    let input_file_path = Path::new(input_file);
    let interactor_path = Path::new(interactor);

    if !path.exists() || !input_file_path.exists() || !interactor_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "code, input or interactor not found",
        )
        .into());
    }

    let input = fs::read_to_string(input_file_path)?;

    let judge = InteractiveJudge::new(
        exec.into(),
        input,
        limit.clone(),
        interactor.into(),
    );
    let judge_result = judge.judge()?;

    Ok(judge_result)
}

pub fn get_path_of_tankcell()->String{
    std::env::current_exe().unwrap().parent().unwrap().join("tank_cell").to_string_lossy().to_string()
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
        assert!(result.time.unwrap() > 0 && result.time.unwrap() <= 30 * 1000);
        assert!(result.memory.unwrap() > 0 && result.memory.unwrap() <= 256 * 1024);

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

        assert!(result.time.unwrap() > 1000);
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
        assert!(result.memory.unwrap() > 256 * 1024);
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
    fn interactive_accept() -> Result<()> {
        let judge = InteractiveJudge {
            exec: "./test_dep/interactive/solution".into(),
            input: "???".into(),
            limit: LimitConfig {
                time_limit: 1000,
                memory_limit: 256,
            },
            interactor: "./test_dep/interactive/interactor".into(),
        };
        let result = judge.judge()?;
        println!("{:?}", result.status);

        Ok(())
    }
}
