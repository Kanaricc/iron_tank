use std::fs;

use compile::Compiler;
use tank_core::{
    compile::{self, CompileResult},
    problem::ProblemConfig,
    error::{ Result},
    JudgeStatus,
};

#[test]
fn normal_accept() -> Result<()> {
    let compiler = compile::gpp::CompilerGPP::new().unwrap();
    let program = compiler.compile(fs::read_to_string("../test_dep/normal/times2.cpp").unwrap());
    assert!(matches!(program, CompileResult::OK(_)));
    let program = match program {
        compile::CompileResult::OK(program) => program,
        _ => unreachable!(),
    };
    let judge = ProblemConfig::from_file("../test_dep/normal/problem.yaml")?;

    let result = &judge.judge(program)?[0];

    assert!(matches!(result.status, JudgeStatus::Accept));
    assert!(result.time.unwrap() > 0 && result.time.unwrap() <= 30 * 1000);
    assert!(result.memory.unwrap() > 0 && result.memory.unwrap() <= 256 * 1024);

    Ok(())
}

#[test]
fn normal_wrong_answer() -> Result<()> {
    let compiler = compile::gpp::CompilerGPP::new().unwrap();
    let program = compiler.compile(fs::read_to_string("../test_dep/normal/times3.cpp").unwrap());
    assert!(matches!(program, CompileResult::OK(_)));
    let program = match program {
        compile::CompileResult::OK(program) => program,
        _ => unreachable!(),
    };
    let judge = ProblemConfig::from_file("../test_dep/normal/problem.yaml")?;

    let result = &judge.judge(program)?[0];

    assert!(matches!(result.status, JudgeStatus::WrongAnswer));

    Ok(())
}

#[test]
fn normal_time_limit_exceeded() -> Result<()> {
    let compiler = compile::gpp::CompilerGPP::new().unwrap();
    let program = compiler.compile(fs::read_to_string("../test_dep/normal/tle.cpp").unwrap());
    assert!(matches!(program, CompileResult::OK(_)));
    let program = match program {
        compile::CompileResult::OK(program) => program,
        _ => unreachable!(),
    };
    let judge = ProblemConfig::from_file("../test_dep/normal/problem.yaml")?;

    let result = &judge.judge(program)?[0];

    println!("{:#?}",result);

    assert!(result.time.unwrap() > 1000);
    debug_assert!(matches!(result.status, JudgeStatus::TimeLimitExceeded));
    Ok(())
}

#[test]
fn normal_memory_limit_exceeded() -> Result<()> {
    let compiler = compile::gpp::CompilerGPP::new().unwrap();
    let program = compiler.compile(fs::read_to_string("../test_dep/normal/mle.cpp").unwrap());
    assert!(matches!(program, CompileResult::OK(_)));
    let program = match program {
        compile::CompileResult::OK(program) => program,
        _ => unreachable!(),
    };
    let judge = ProblemConfig::from_file("../test_dep/normal/problem.yaml")?;

    let result = &judge.judge(program)?[0];

    assert!(matches!(result.status, JudgeStatus::MemoryLimitExceeded));
    assert!(result.memory.unwrap() > 256 * 1024);
    Ok(())
}

#[test]
#[should_panic]
fn normal_invalid_path() {
    let compiler = compile::gpp::CompilerGPP::new().unwrap();
    let program = compiler.compile(fs::read_to_string("../test_dep/normal/?!?!.cpp").unwrap());
    assert!(matches!(program, CompileResult::OK(_)));
    let program = match program {
        compile::CompileResult::OK(program) => program,
        _ => unreachable!(),
    };
    let judge = ProblemConfig::from_file("../test_dep/normal/problem.yaml").unwrap();

    let _result = judge.judge(program).unwrap();
}
