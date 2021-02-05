use crate::JudgeStatus;

pub enum CompareResult {
    Same,
    Different,
    PatternDifferent,
}

impl Into<JudgeStatus> for CompareResult{
    fn into(self) -> JudgeStatus {
        match self {
            CompareResult::Same => JudgeStatus::Accept,
            CompareResult::Different => JudgeStatus::WrongAnswer,
            CompareResult::PatternDifferent => JudgeStatus::PresentationError,
        }
    }
}

pub trait CompareMode {
    fn compare(&self,str1: &String, str2: &String) -> CompareResult;
}

pub struct GlobalCompare;

impl CompareMode for GlobalCompare {
    fn compare(&self,str1: &String, str2: &String) -> CompareResult {
        let value_res=ValueCompare{}.strict_compare(str1, str2);

        if str1 == str2 {
            CompareResult::Same
        } else {
            value_res
        }
    }
}

pub struct LineCompare;

impl CompareMode for LineCompare {
    fn compare(&self,str1: &String, str2: &String) -> CompareResult {
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

        return CompareResult::Same;
    }
}

pub struct ValueCompare;

impl ValueCompare{
    fn strict_compare(&self,str1:&String,str2:&String)->CompareResult{
        if str1==str2{
            return CompareResult::Same;
        }else{
            match self.compare(str1, str2) {
                CompareResult::Same => CompareResult::PatternDifferent,
                CompareResult::Different => CompareResult::Different,
                _ => unreachable!(),
            }
        }
    }
}

impl CompareMode for ValueCompare {
    fn compare(&self,str1: &String, str2: &String) -> CompareResult {
        let str1 = str1.replace("\n", "").replace(" ", "");
        let str2 = str2.replace("\n", "").replace(" ", "");

        if str1==str2{
            CompareResult::Same
        }else{
            CompareResult::Different
        }
    }
}
