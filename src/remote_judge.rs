use crate::error::Result;
use crate::JudgeResult;
use crate::{error::Error, JudgeStatus};
use regex::Regex;
use reqwest::{blocking::multipart::Form, blocking::Client};
use select::{
    document::Document,
    predicate::{self, Predicate},
};
use std::fs;

pub trait RemoteJudge {
    type Jar;

    fn get_name(&self) -> String;
    /// Prepare for remote judge
    ///
    /// * Login
    /// * ......
    ///
    /// # Reason for this function being sync
    /// The singleton nature of the remote state prevents this function from being concurrent.
    fn prepare(&mut self) -> Result<()>;

    fn load(&mut self, jar: Self::Jar) -> Result<()>;
    fn persist(&mut self) -> Self::Jar;

    /// Request for judge, returning remote sniffer which producing result.
    ///
    /// RemoteJudge will be consumed.
    ///
    /// # Reason for this function being sync
    /// The singleton nature of the remote state prevents this function from being concurrent.
    ///
    /// Untill current request is done (submited and queried the remote number),
    /// no remote judge are permitted to create new client.
    fn judge(self) -> Result<Box<dyn RemoteSniffer>>;

    fn make_error(&self, msg: &str) -> Error {
        Error::Judge {
            judge_name: self.get_name(),
            msg: msg.into(),
        }
    }
}

pub trait RemoteSniffer {
    fn fetch_status(&self) -> Result<JudgeResult>;
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

#[allow(dead_code)]
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
    type Jar = String;

    fn get_name(&self) -> String {
        "Opentrains Judge".into()
    }

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

    fn load(&mut self, jar: Self::Jar) -> Result<()> {
        self.sid = Some(jar);

        Ok(())
    }

    fn persist(&mut self) -> Self::Jar {
        self.sid.clone().unwrap()
    }

    fn judge(self) -> Result<Box<dyn RemoteSniffer>> {
        if let None = self.sid {
            return Err(self.make_error("No session set. Please login first."));
        }
        let temp_dir = tempfile::TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("code.dat");
        fs::write(&temp_file, self.src.as_bytes()).unwrap();
        let temp_file = temp_file.to_str().unwrap().to_string();
        let sid = self.sid.clone().unwrap();
        let form = Form::new()
            .text("SID", sid.clone())
            .text("prob_id", self.problem_id.to_string())
            .text("lang_id", self.language_id.to_string())
            .text("action_40", "Send!")
            .file("file", temp_file)?;

        let res = self
            .client
            .post("http://opentrains.snarknews.info/~ejudge/team.cgi")
            .multipart(form)
            .send()?;
        let res = res.text()?;

        let res = Document::from(res.as_str());

        let res = res
            .find(predicate::Attr("id", "l13").descendant(predicate::Name("tr")))
            .nth(1)
            .unwrap();
        let res = res.find(predicate::Name("td")).next().unwrap();

        Ok(Box::new(OpentrainsJudgeSniffer {
            sid,
            run_id: res.text().parse().unwrap(),
        }))
    }

    fn make_error(&self, msg: &str) -> Error {
        Error::Judge {
            judge_name: self.get_name(),
            msg: msg.into(),
        }
    }
}

pub struct OpentrainsJudgeSniffer {
    sid: String,
    run_id: u32,
}

impl RemoteSniffer for OpentrainsJudgeSniffer {
    fn fetch_status(&self) -> Result<JudgeResult> {
        let url = format!(
            "http://opentrains.snarknews.info/~ejudge/team.cgi?SID={}&run_id={}&action=37",
            self.sid, self.run_id
        );

        let content = reqwest::blocking::get(&url)?.text()?;
        let sel = Document::from(content.as_str());

        if let Some(sel) = sel
            .find(predicate::Name("h2").descendant(predicate::Name("font")))
            .nth(0)
        {
            let sel = sel.text();

            // TODO: status not covered completely
            // refer to https://ejudge.ru/wiki/index.php/%D0%92%D0%B5%D1%80%D0%B4%D0%B8%D0%BA%D1%82%D1%8B_%D1%82%D0%B5%D1%81%D1%82%D0%B8%D1%80%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D1%8F
            let status = match sel.as_str() {
                "OK" => JudgeStatus::Accept,
                "Wrong answer" => JudgeStatus::WrongAnswer,
                "Time-limit exceeded"=>JudgeStatus::TimeLimitExceeded,
                "Memory-limit exceeded"=>JudgeStatus::MemoryLimitExceeded,
                "Wall time-limit exceeded"=>JudgeStatus::InteractionTimeLimitExceeded,
                "Run-time error"=>JudgeStatus::RuntimeError,
                _ => unreachable!(),
            };

            return Ok(JudgeResult {
                status: status,
                time: None,
                memory: None,
                stdout: None,
                stderr: None,
            });
        } else if let Some(_sel) = sel.find(predicate::Name("pre")).next() {
            // TODO: add CE info
            return Ok(JudgeResult {
                status: JudgeStatus::ComplierError,
                time: None,
                memory: None,
                stdout: None,
                stderr: None,
            });
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use select::{
        document::Document,
        predicate::{self, Predicate},
    };

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

    #[test]
    fn fetch_remote_number() -> Result<()> {
        let res = reqwest::blocking::get(
            "http://opentrains.snarknews.info/~ejudge/team.cgi?SID=d858b4b226d4dcff&action=140",
        )?
        .text()?;

        let test = Document::from(res.as_str());

        let test = test
            .find(predicate::Attr("id", "l13").descendant(predicate::Name("tr")))
            .nth(1)
            .unwrap();
        let test = test.find(predicate::Name("td")).next().unwrap();
        println!("{:#?}", test.text());

        Ok(())
    }

    #[test]
    fn fetch_remote_result() -> Result<()> {
        let sniffer = OpentrainsJudgeSniffer {
            sid: "d858b4b226d4dcff".into(),
            run_id: 332,
        };

        sniffer.fetch_status();
        Ok(())
    }
}
