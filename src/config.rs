use serde::{Serialize,Deserialize};

use crate::compare::{ComparisionMode, GlobalCompare, LineCompare, ValueCompare};

#[derive(Debug,Serialize,Deserialize)]
pub struct LimitConfig{
    pub time_limit:u64,
    pub memory_limit:u64,
}
#[derive(Debug,Serialize,Deserialize)]
pub struct CaseConfig{
    pub inputfile_path:String,
    pub answerfile_path:String,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ComparisionModeConfig{
    Full,Line,Value
}

impl Into<Box<dyn ComparisionMode>> for ComparisionModeConfig {
    fn into(self) -> Box<dyn ComparisionMode> {
        match self {
            ComparisionModeConfig::Full => Box::new(GlobalCompare{}),
            ComparisionModeConfig::Line => Box::new(LineCompare{}),
            ComparisionModeConfig::Value => Box::new(ValueCompare{}),
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ProblemConfig{
    pub name:String,
    pub limit_config:LimitConfig,
    pub comparision_mode:ComparisionModeConfig,
    pub cases:Vec<CaseConfig>,
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn serialize(){
        let problem=ProblemConfig{
            name: "A".into(),
            limit_config: LimitConfig{
                time_limit: 1,
                memory_limit: 2,
            },
            comparision_mode: ComparisionModeConfig::Line,
            cases: vec![CaseConfig{
                inputfile_path:"in".into(),
                answerfile_path:"out".into(),
            }],
        };
        let s=serde_yaml::to_string(&problem).unwrap();
        println!("{}",s);
    }
}