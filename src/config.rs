use serde::{Serialize,Deserialize};

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
pub enum ComparisionMode{
    Full,Line,Value
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ProblemConfig{
    pub name:String,
    pub limit_config:LimitConfig,
    pub comparision_mode:ComparisionMode,
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
            comparision_mode: ComparisionMode::Line,
            cases: vec![CaseConfig{
                inputfile_path:"in".into(),
                answerfile_path:"out".into(),
            }],
        };
        let s=serde_yaml::to_string(&problem).unwrap();
        println!("{}",s);
    }
}