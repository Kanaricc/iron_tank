use serde::{Deserialize, Serialize};

use crate::{
    compare::{ComparisionMode, GlobalCompare, LineCompare, ValueCompare},
    error::Result,
    judge::{launch_normal_case_judge, launch_special_case_judge},
    JudgeResult,
};
use std::fs;
use std::path::Path;
#[derive(Debug, Serialize, Deserialize)]
pub struct LimitConfig {
    pub time_limit: u64,
    pub memory_limit: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CaseConfig {
    pub inputfile_path: String,
    pub answerfile_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComparisionModeConfig {
    Full,
    Line,
    Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JudgeModeConfig {
    Normal {
        comparision_mode: ComparisionModeConfig,
    },
    Special {
        checker: String,
    },
}

impl Into<Box<dyn ComparisionMode>> for &ComparisionModeConfig {
    fn into(self) -> Box<dyn ComparisionMode> {
        match self {
            ComparisionModeConfig::Full => Box::new(GlobalCompare {}),
            ComparisionModeConfig::Line => Box::new(LineCompare {}),
            ComparisionModeConfig::Value => Box::new(ValueCompare {}),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemConfig {
    pub name: String,
    #[serde(skip_serializing,skip_deserializing)]
    path: String,
    pub limit_config: LimitConfig,
    pub judge_mode: JudgeModeConfig,
    pub cases: Vec<CaseConfig>,
}

impl ProblemConfig {

    fn from_string(content:&str)->Result<Self>{
        let v: Self = serde_yaml::from_str(&content).unwrap();
        Ok(v)
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).unwrap();
        let mut v=Self::from_string(&content)?;

        let r_path=Path::new(path).canonicalize()?;
        let r_path=r_path.parent().unwrap();
        let r_path=r_path.canonicalize()?.to_string_lossy().to_string();
        v.path=r_path;

        v.check_valid()?;
        Ok(v)
    }

    fn check_valid(&self)->Result<()>{
        for case in self.cases.iter() {
            if !Path::new(&self.find_relative_path(&case.inputfile_path)).exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("input file `{}` not found",case.inputfile_path),
                )
                .into());
            }
            if let Some(answerfile_path)=&case.answerfile_path {
                if !Path::new(&self.find_relative_path(answerfile_path)).exists(){
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("answer file `{}` not found",answerfile_path),
                    )
                    .into());
                }
            }
        }

        if let JudgeModeConfig::Special { checker }=&self.judge_mode{
            if !Path::new(&self.find_relative_path(checker)).exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("checker `{}` not found",checker),
                )
                .into());
            }
        }

        Ok(())
    }
    pub fn find_relative_path(&self,path:&str)->String{
        let t=Path::new(&self.path).join(path).to_string_lossy().to_string();
        t
    }
    pub fn judge(&self, exec: &str) -> Result<Vec<JudgeResult>> {
        let exec_path = Path::new(exec);
        if !exec_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "exec not found").into());
        }

        let mut judge_results = Vec::new();

        for case in self.cases.iter() {
            let judge_result = match &self.judge_mode {
                JudgeModeConfig::Normal { comparision_mode } => launch_normal_case_judge(
                    exec,
                    self.find_relative_path(&case.inputfile_path).as_str(),
                    self.find_relative_path(&case.answerfile_path.as_ref().unwrap()).as_str(),
                    &self.limit_config,
                    comparision_mode,
                ),
                JudgeModeConfig::Special { checker } => launch_special_case_judge(
                    &exec,
                    self.find_relative_path(&case.inputfile_path).as_str(),
                    self.find_relative_path(&checker).as_str(),
                    &self.limit_config,
                ),
            }?;

            judge_results.push(judge_result);
        }

        Ok(judge_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize() {
        let problem = ProblemConfig {
            name: "A".into(),
            limit_config: LimitConfig {
                time_limit: 1,
                memory_limit: 2,
            },
            judge_mode: JudgeModeConfig::Normal {
                comparision_mode: ComparisionModeConfig::Line,
            },
            cases: vec![CaseConfig {
                inputfile_path: "in".into(),
                answerfile_path: "out".to_string().into(),
            }],
            path: "./test_dep/problem".into(),
        };
        let s = serde_yaml::to_string(&problem).unwrap();
        println!("{}", s);
    }

    #[test]
    fn deserialize(){
        let _problem=ProblemConfig::from_file("./test_dep/problem/problem.yaml").unwrap();
    }

    #[test]
    fn multi_cases_judge(){
        let problem=ProblemConfig::from_file("./test_dep/problem/problem.yaml").unwrap();
        println!("{:#?}",problem.judge("./test_dep/normal"));
    }
}
