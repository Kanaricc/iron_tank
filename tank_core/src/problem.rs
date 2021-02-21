use serde::{Deserialize, Serialize};

use crate::{
    compare::{ComparisionMode, GlobalCompare, LineCompare, ValueCompare},
    compile::CompiledProgram,
    error::{Error, Result},
    judge::{launch_interactive_case_judge, launch_normal_case_judge, launch_special_case_judge},
    lint::DataLinter,
    JudgeResult,
};
use std::fs;
use std::path::Path;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "limitConfig")]
pub struct LimitConfig {
    #[serde(rename = "timeLimit")]
    pub time_limit: u64,
    #[serde(rename = "memoryLimit")]
    pub memory_limit: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CaseConfig {
    #[serde(rename = "inputFile")]
    pub inputfile_path: String,
    #[serde(rename = "answerFile")]
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
        #[serde(rename = "comparisionMode")]
        comparision_mode: ComparisionModeConfig,
    },
    Special {
        checker: String,
    },
    Interactive {
        interactor: String,
        has_input: bool,
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

#[derive(Serialize, Deserialize)]
pub struct ProblemConfig<'a> {
    pub name: String,
    #[serde(skip_serializing, skip_deserializing)]
    path: String,
    #[serde(rename = "limitConfig")]
    pub limit_config: LimitConfig,
    #[serde(rename = "judgeMode")]
    pub judge_mode: JudgeModeConfig,
    #[serde(rename = "inputLint")]
    pub input_lint: Option<DataLinter<'a>>,
    #[serde(rename = "answerLint")]
    pub answer_lint: Option<DataLinter<'a>>,
    pub cases: Vec<CaseConfig>,
}

impl<'a> ProblemConfig<'a> {
    fn from_string(content: &str) -> Result<Self> {
        let mut v: Self = serde_yaml::from_str(&content).unwrap();
        if let Some(lint) = &mut v.input_lint {
            lint.init()?;
        }
        if let Some(lint) = &mut v.answer_lint {
            lint.init()?;
        }
        Ok(v)
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).unwrap();
        let mut v = Self::from_string(&content)?;

        let r_path = Path::new(path).canonicalize()?;
        let r_path = r_path.parent().unwrap();
        let r_path = r_path.canonicalize()?.to_string_lossy().to_string();
        v.path = r_path;

        v.check_valid()?;
        Ok(v)
    }

    pub fn lint_data(&self) -> Result<Vec<String>> {
        let mut lint_res = vec![];
        for case in self.cases.iter() {
            if !Path::new(&self.find_relative_path(&case.inputfile_path)).exists() {
                return Err(Error::NotFound(case.inputfile_path.to_string()));
            }
            if let Some(answerfile_path) = &case.answerfile_path {
                if !Path::new(&self.find_relative_path(answerfile_path)).exists() {
                    return Err(Error::NotFound(answerfile_path.to_string()));
                }
            }

            if let Some(lint) = &self.input_lint {
                let res = lint.check(&fs::read(self.find_relative_path(&case.inputfile_path))?);
                if res.len() > 0 {
                    lint_res.extend(
                        res.into_iter()
                            .map(|f| format!("{}: {}", case.inputfile_path, f)),
                    );
                }
            }
            if let Some(lint) = &self.answer_lint {
                if let Some(answerfile_path) = &case.answerfile_path {
                    let res = lint.check(&fs::read(self.find_relative_path(answerfile_path))?);
                    if res.len() > 0 {
                        lint_res.extend(
                            res.into_iter()
                                .map(|f| format!("{}: {}", answerfile_path, f)),
                        );
                    }
                }
            }
        }

        Ok(lint_res)
    }

    fn check_valid(&self) -> Result<()> {
        for case in self.cases.iter() {
            if !Path::new(&self.find_relative_path(&case.inputfile_path)).exists() {
                return Err(Error::NotFound(case.inputfile_path.to_string()));
            }
            if let Some(answerfile_path) = &case.answerfile_path {
                if !Path::new(&self.find_relative_path(answerfile_path)).exists() {
                    return Err(Error::NotFound(answerfile_path.to_string()));
                }
            }
        }

        if let JudgeModeConfig::Special { checker } = &self.judge_mode {
            if !Path::new(&self.find_relative_path(checker)).exists() {
                    return Err(Error::NotFound(checker.to_string()));
            }
        }

        Ok(())
    }
    pub fn find_relative_path(&self, path: &str) -> String {
        let t = Path::new(&self.path)
            .join(path)
            .to_string_lossy()
            .to_string();
        t
    }
    pub fn judge(&self, exec: CompiledProgram) -> Result<Vec<JudgeResult>> {
        let mut judge_results = Vec::new();

        for case in self.cases.iter() {
            let judge_result = match &self.judge_mode {
                JudgeModeConfig::Normal { comparision_mode } => launch_normal_case_judge(
                    exec.clone(),
                    self.find_relative_path(&case.inputfile_path).as_str(),
                    self.find_relative_path(&case.answerfile_path.as_ref().unwrap())
                        .as_str(),
                    self.limit_config.clone(),
                    comparision_mode,
                ),
                JudgeModeConfig::Special { checker } => launch_special_case_judge(
                    exec.clone(),
                    self.find_relative_path(&case.inputfile_path).as_str(),
                    self.find_relative_path(&checker).as_str(),
                    self.limit_config.clone(),
                ),
                JudgeModeConfig::Interactive {
                    interactor,
                    has_input,
                } => {
                    let input = if has_input.clone() {
                        Some(self.find_relative_path(&case.inputfile_path))
                    } else {
                        None
                    };
                    launch_interactive_case_judge(
                        exec.clone(),
                        input,
                        self.find_relative_path(&interactor).as_str(),
                        self.limit_config.clone(),
                    )
                }
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
            path: "../test_dep/problem".into(),
            input_lint: None,
            answer_lint: None,
        };
        let s = serde_yaml::to_string(&problem).unwrap();
        println!("{}", s);
    }

    #[test]
    fn deserialize() -> Result<()> {
        let _problem = ProblemConfig::from_file("../test_dep/normal/problem.yaml")?;

        Ok(())
    }
}
