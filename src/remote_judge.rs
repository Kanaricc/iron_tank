use regex::Regex;
use reqwest::{blocking::multipart::Form, blocking::Client};
use crate::error::Error;
use crate::error::Result;
use crate::JudgeResult;
use std::fs;

pub trait RemoteJudge {
    fn get_name(&self) -> String;
    fn prepare(&mut self) -> Result<()>;
    fn judge(self) -> Result<JudgeResult>;

    fn make_error(&self, msg: &str) -> Error {
        Error::Judge {
            judge_name: self.get_name(),
            msg: msg.into(),
        }
    }
}
struct OpentrainsJudge {
    username: String,
    password: String,
    sid: Option<String>,
    contest_id: String,
    problem_id: u32,
    language_id: u32,
    src: String,

    client: Client,
}

impl OpentrainsJudge {
    pub fn new(
        username: String,
        password: String,
        contest_id: String,
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
        let rgx = Regex::new(r#"SID="(.+)""#).unwrap();
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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("code.dat");
        fs::write(&temp_file, self.src.as_bytes()).unwrap();
        let temp_file = temp_file.to_str().unwrap().to_string();
        let sid = self.sid.unwrap();
        let form = Form::new()
            .text("SID", sid)
            .text("prob_id", self.problem_id.to_string())
            .text("lang_id", self.language_id.to_string())
            .text("action_40", "Send!")
            .file("file", temp_file)?;

        let res = self
            .client
            .post("http://opentrains.snarknews.info/~ejudge/team.cgi")
            .multipart(form)
            .send()?;
        let res=res.text()?;
        
        println!("{}",res);

        todo!()
    }

    fn get_name(&self) -> String {
        "Opentrains Judge".into()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn opentrains_judge() -> Result<()> {
        let mut judge = OpentrainsJudge::new(
            "username".into(),
            "password".into(),
            "010513".into(),
            1,
            6,
            "src".into(),
        );
        judge.prepare()?;

        println!("{:?}", judge.sid);

        judge.judge()?;

        Ok(())
    }
}