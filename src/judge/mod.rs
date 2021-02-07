mod interactive;
mod normal;
mod special;

use std::{fs, path::Path};

use self::{interactive::InteractiveJudge, normal::NormalJudge, special::SpecialJudge};
use crate::{
    compare::ComparisionMode,
    config::{ComparisionModeConfig, LimitConfig},
    error::Result,
    JudgeResult,
};

pub trait Judge {
    fn judge(self) -> Result<JudgeResult>;
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
    input_file: Option<String>,
    interactor: &str,
    limit: &LimitConfig,
) -> Result<JudgeResult> {
    let path = Path::new(exec);

    let interactor_path = Path::new(interactor);

    if !path.exists() || !interactor_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "code, input or interactor not found",
        )
        .into());
    }

    let input = if let Some(input_file) = input_file {
        let input_file_path = Path::new(&input_file);
        if !input_file_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "code, input or interactor not found",
            )
            .into());
        }

        Some(fs::read_to_string(input_file_path)?)
    } else {
        None
    };
    let judge = InteractiveJudge::new(exec.into(), input, limit.clone(), interactor.into());
    let judge_result = judge.judge()?;

    Ok(judge_result)
}

pub fn get_path_of_tankcell() -> String {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("tank_cell")
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compare::ValueCompare, JudgeStatus};

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
        let judge = InteractiveJudge::new(
            "./test_dep/interactive/solution".into(),
            "???".to_string().into(),
            LimitConfig {
                time_limit: 1000,
                memory_limit: 256,
            },
            "./test_dep/interactive/interactor".into(),
        );
        let result = judge.judge()?;
        println!("{:?}", result.status);

        Ok(())
    }
}
