use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

use crate::{JudgeResult, JudgeStatus, compile::CompiledProgram, config::LimitConfig, error::Error, error::Result, probe::ProcessProbe};

use super::{get_path_of_tankcell, Judge};

use std::io::{Write,Read};

use libc::*;

pub struct InteractiveJudge {
    program: CompiledProgram,
    input: Option<String>,
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
    pub fn new(program: CompiledProgram, input: Option<String>, limit: LimitConfig, interactor: String) -> Self {
        Self {
            program,
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

        let interactor=if let Some(input)=self.input{
            Command::new(interactor_fullpath)
            .arg(input)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
        }else{
            Command::new(interactor_fullpath)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
        };

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

        let path = Path::new(&self.program.path);
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
            .arg("--")
            .args(self.program.args)
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

        // channel switching inputs and outputs of interactor and user program
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
                    Err(err) => {
                        if let std::io::ErrorKind::BrokenPipe = err.kind(){
                            // it seems like this error can be regarded that the program quits.
                            csender.send(InteractiveMessage::UserQuit).unwrap();
                        }else{
                            panic!("error when trying reading from user program: {}",err);
                        }
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
                        if let std::io::ErrorKind::BrokenPipe = err.kind(){
                            // it seems like this error can be regarded that the program quits.
                            isender.send(InteractiveMessage::InteractorQuit).unwrap();
                        }else{
                            panic!("error when trying reading from interactor: {}",err);
                        }
                        println!("interactor reading error: {:?}", err);
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
                        if let Err(err) = iin.write_all(buf.as_slice()) {
                            if let std::io::ErrorKind::BrokenPipe = err.kind(){
                                break;
                            }else{
                                panic!("error when trying writing to interactor: {}",err);
                            }
                        }
                    }
                    InteractiveMessage::InteractorOut(buf) => {
                        b_input
                            .lock()
                            .unwrap()
                            .push(std::string::String::from_utf8(buf.clone()).unwrap());
                        if let Err(err) = cin.write_all(buf.as_slice()) {
                            if let std::io::ErrorKind::BrokenPipe = err.kind(){
                                break;
                            }else{
                                panic!("error when trying writing to user program: {}",err);
                            }
                        }
                    }
                    InteractiveMessage::UserQuit => {
                        break;
                    }
                    InteractiveMessage::InteractorQuit => {
                        break;
                    }
                }
            }
        });

        // wait for user quitting
        // need not to kill user program when interactor quits first. it is seen as TLE
        let probe_res = probe.watching();
        // notice broker just in case
        sender.send(InteractiveMessage::UserQuit).unwrap();
        // interactor MUST quit before user, or it will be killed
        unsafe {
            // maybe there is a more elegant way to do it
            kill(interactor_pid as i32, SIGKILL);
        }
        // wait for broker to quit
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
