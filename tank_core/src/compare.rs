use crate::JudgeStatus;

pub enum ComparisionResult {
    Same,
    Different,
    PatternDifferent,
}

impl Into<JudgeStatus> for ComparisionResult{
    fn into(self) -> JudgeStatus {
        match self {
            ComparisionResult::Same => JudgeStatus::Accept,
            ComparisionResult::Different => JudgeStatus::WrongAnswer,
            ComparisionResult::PatternDifferent => JudgeStatus::PresentationError,
        }
    }
}

pub trait ComparisionMode {
    fn compare(&self,str1: &String, str2: &String) -> ComparisionResult;
}

pub struct GlobalCompare;

impl ComparisionMode for GlobalCompare {
    fn compare(&self,str1: &String, str2: &String) -> ComparisionResult {
        let value_res=ValueCompare{}.strict_compare(str1, str2);

        if str1 == str2 {
            ComparisionResult::Same
        } else {
            value_res
        }
    }
}

pub struct LineCompare;

impl ComparisionMode for LineCompare {
    fn compare(&self,str1: &String, str2: &String) -> ComparisionResult {
        let value_res=ValueCompare{}.strict_compare(str1, str2);

        let str1 = str1.trim();
        let str2 = str2.trim();
        let str1: Vec<&str> = str1.split("\n").map(|f| f.trim_end()).collect();
        let str2: Vec<&str> = str2.split("\n").map(|f| f.trim_end()).collect();

        if str1.len() != str2.len() {
            return value_res;
        }

        for pair in str1.into_iter().zip(str2.into_iter()) {
            if pair.0 != pair.1 {
                return value_res;
            }
        }

        return ComparisionResult::Same;
    }
}

pub struct ValueCompare;

impl ValueCompare{
    fn strict_compare(&self,str1:&String,str2:&String)->ComparisionResult{
        if str1==str2{
            return ComparisionResult::Same;
        }else{
            match self.compare(str1, str2) {
                ComparisionResult::Same => ComparisionResult::PatternDifferent,
                ComparisionResult::Different => ComparisionResult::Different,
                _ => unreachable!(),
            }
        }
    }
}

impl ComparisionMode for ValueCompare {
    fn compare(&self,str1: &String, str2: &String) -> ComparisionResult {
        let str1 = str1.replace("\n", "").replace(" ", "");
        let str2 = str2.replace("\n", "").replace(" ", "");

        if str1==str2{
            ComparisionResult::Same
        }else{
            ComparisionResult::Different
        }
    }
}