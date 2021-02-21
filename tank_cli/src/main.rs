use core::panic;
use std::{fs, path::Path};

use clap::Clap;
use compile::CompilerDescriptor;
use tank_core::{compile::{self, CompileResult, CompiledProgram, Compiler}, error::{Error, Result}};
use tank_core::{
    problem::{ComparisionModeConfig, LimitConfig, ProblemConfig},
    judge::{launch_interactive_case_judge, launch_normal_case_judge, launch_special_case_judge},
};
#[derive(Clap)]
#[clap(
    version = "0.3.0",
    name = "Iron Tank",
    author = "Kanari",
    about = "A fast and reliable judge container wrtten in Rust."
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "0.2.0", about = "Judge in normal mode")]
    Compile(CompileConfig),
    #[clap(version = "0.2.0", about = "Judge in normal mode")]
    Normal(NormalJudgeConfig),
    #[clap(version = "0.2.0", about = "Judge in special mode")]
    Special(SpecialJudgeConfig),
    #[clap(version = "0.2.0", about = "Judge in interactive mode")]
    Interactive(InteractiveJudgeConfig),
    #[clap(version = "0.2.0", about = "Judge using config.yaml")]
    Prefab(PrefabJudgeConfig),
    #[clap(version = "0.1.0", about = "Lint problem using config.yaml")]
    Lint(LintConfig),
    #[clap(version = "0.1.0", about = "Debug mode")]
    Debug,
}

#[derive(Clap, Debug)]
struct CompileConfig{
    #[clap(about = "path of source")]
    file:String,
}

#[derive(Clap, Debug)]
struct NormalJudgeConfig {
    #[clap(about = "path of code")]
    src_path: String,
    #[clap(short, about = "input file path")]
    input_file: String,
    #[clap(short, about = "answer file path")]
    answer_file: String,
    #[clap(short, default_value = "1024", about = "memory limit(MB)")]
    memory_limit: u64,
    #[clap(short, default_value = "30000", about = "time limit(MS)")]
    time_limit: u64,
    #[clap(
        short,
        default_value = "line",
        about = "compare method: full, line, value.\nrefer to document for more details."
    )]
    compare_method: String,
}

#[derive(Clap, Debug)]
struct SpecialJudgeConfig {
    #[clap(about = "checker program path")]
    checker: String,
    #[clap(about = "path of code")]
    src_path: String,
    #[clap(short, about = "input file path")]
    input_file: String,
    #[clap(short, default_value = "1024", about = "memory limit(MB)")]
    memory_limit: u64,
    #[clap(short, default_value = "30000", about = "time limit(MS)")]
    time_limit: u64,
}

#[derive(Clap, Debug)]
struct InteractiveJudgeConfig {
    #[clap(about = "interactor code")]
    interactor: String,
    #[clap(about = "path of program to run")]
    src_path: String,
    #[clap(short, about = "input file path")]
    input_file: Option<String>,
    #[clap(short, default_value = "1024", about = "memory limit(MB)")]
    memory_limit: u64,
    #[clap(short, default_value = "30000", about = "time limit(MS)")]
    time_limit: u64,
}

#[derive(Clap, Debug)]
struct PrefabJudgeConfig {
    #[clap(about = "problem config")]
    config: String,
    #[clap(about = "path of code")]
    src_path: String,
}

#[derive(Clap, Debug)]
struct LintConfig {
    #[clap(about = "problem config")]
    config: String,
}

#[derive(Clap, Debug)]
struct DebugJudge {}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Normal(config) => {
            let comparision_mode = match config.compare_method.as_str() {
                "full" => ComparisionModeConfig::Full,
                "line" => ComparisionModeConfig::Line,
                "value" => ComparisionModeConfig::Value,
                _ => Err(Error::Argument("comparation mode not found".into()))?,
            };

            let compiler=compile(&config.src_path);

            let judge_result = launch_normal_case_judge(
                compiler.1,
                &config.input_file,
                &config.answer_file,
                LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
                &comparision_mode,
            )?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Special(config) => {
            let compiler=compile(&config.src_path);

            let judge_result = launch_special_case_judge(
                compiler.1,
                &config.input_file,
                &config.checker,
                LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
            )?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Prefab(config) => {
            let compiler=compile(&config.src_path);
            
            // TOOD: judge should use compiledprogram instead of str
            let judge_result = ProblemConfig::from_file(&config.config)?.judge(compiler.1)?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Interactive(config) => {
            let compiler=compile(&config.src_path);
            
            let judge_result = launch_interactive_case_judge(
                compiler.1,
                config.input_file,
                &config.interactor,
                LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
            );
            println!("{:#?}", judge_result);
        }
        SubCommand::Debug => {}
        SubCommand::Compile(config) => {
            let _compiler=compile(&config.file);
        }
        
        SubCommand::Lint(config) => {
            let judge_result = ProblemConfig::from_file(&config.config)?;
            println!("{:#?}", judge_result.lint_data()?);
        }
    }

    Ok(())
}

fn compile(file:&str)->(Box<dyn Compiler>,CompiledProgram){
    let path=Path::new(file);
    let src=fs::read_to_string(path.canonicalize().unwrap()).unwrap();
    let extension=path.extension().unwrap().to_str().unwrap();

    let compiler:Box<dyn Compiler>;
    let result;
    match extension {
        _ if compile::gpp::CompilerGPP::support_sufix().contains(&extension)=>{
            compiler=Box::new(compile::gpp::CompilerGPP::new().unwrap());
            result=compiler.compile(src);
        }
        _ if compile::python::CompilerPython::support_sufix().contains(&extension)=>{
            compiler=Box::new(compile::python::CompilerPython::new().unwrap());
            result=compiler.compile(src);
        }
        _=>unimplemented!()
    }

    if let CompileResult::OK(program)=result{
        return (compiler,program);
    }else{
        panic!("failed to compile file `{}`: {:#?}",file,result);
    }
}