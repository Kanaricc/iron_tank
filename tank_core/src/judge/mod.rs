mod interactive;
mod normal;
mod special;

use std::{fs, path::Path};

use self::{interactive::InteractiveJudge, normal::NormalJudge, special::SpecialJudge};
use crate::{
    compare::ComparisionMode,
    compile::CompiledProgram,
    error::{Error, Result},
    problem::{ComparisionModeConfig, LimitConfig},
    JudgeResult,
};

pub trait Judge {
    fn judge(self) -> Result<JudgeResult>;
}

pub fn launch_normal_case_judge(
    program: CompiledProgram,
    input_file: &str,
    answer_file: &str,
    limit: LimitConfig,
    comparision_mode: &ComparisionModeConfig,
) -> Result<JudgeResult> {
    let path = Path::new(&program.path);
    let input_file_path = Path::new(input_file);
    let answer_file_path = Path::new(answer_file);

    if !path.exists() || !input_file_path.exists() || !answer_file_path.exists() {
        return Err(Error::NotFound(format!("code, input or answer file")));
    }

    let input = fs::read_to_string(input_file_path)?;
    let answer = fs::read_to_string(answer_file_path)?;

    let comparation: Box<dyn ComparisionMode> = comparision_mode.into();

    let judge = NormalJudge::new(
        program,
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
    program: CompiledProgram,
    input_file: &str,
    checker: &str,
    limit: LimitConfig,
) -> Result<JudgeResult> {
    let path = Path::new(&program.path);
    let input_file_path = Path::new(input_file);
    let checker_path = Path::new(checker);

    if !path.exists() || !input_file_path.exists() || !checker_path.exists() {
        return Err(Error::NotFound(format!("code, input or checker file")));
    }

    let input = fs::read_to_string(input_file_path)?;

    let judge = SpecialJudge::new(
        program,
        input,
        limit.memory_limit,
        limit.time_limit,
        checker.into(),
    );
    let judge_result = judge.judge()?;

    Ok(judge_result)
}

pub fn launch_interactive_case_judge(
    program: CompiledProgram,
    input_file: Option<String>,
    interactor: &str,
    limit: LimitConfig,
) -> Result<JudgeResult> {
    let path = Path::new(&program.path);

    let interactor_path = Path::new(interactor);

    if !path.exists() || !interactor_path.exists() {
        return Err(Error::NotFound(format!("code, input or interactor file")));
    }

    let input = if let Some(input_file) = input_file {
        let input_file_path = Path::new(&input_file);
        if !input_file_path.exists() {
            return Err(Error::NotFound(input_file.to_string()));
        }

        Some(fs::read_to_string(input_file_path)?)
    } else {
        None
    };
    let judge = InteractiveJudge::new(program, input, limit, interactor.into());
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
    #[test]
    fn interactive_accept() -> Result<()> {
        let judge = InteractiveJudge::new(
            CompiledProgram::new("../test_dep/interactive/solution".into()),
            "???".to_string().into(),
            LimitConfig {
                time_limit: 1000,
                memory_limit: 256,
            },
            "../test_dep/interactive/interactor".into(),
        );
        let result = judge.judge()?;
        println!("{:?}", result.status);

        Ok(())
    }
}
