use clap::Clap;
use iron_tank::error::{Error, Result};
use iron_tank::{
    config::{ComparisionModeConfig, LimitConfig, ProblemConfig},
    judge::{launch_interactive_case_judge, launch_normal_case_judge, launch_special_case_judge},
};
#[derive(Clap)]
#[clap(
    version = "0.1.1",
    name = "Iron Tank",
    author = "Kanari <iovo7c@gmail.com>",
    about = "A fast and reliable judge container wrtten in Rust."
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "0.1.0", about = "Judge in normal mode")]
    Normal(NormalJudgeConfig),
    #[clap(version = "0.1.0", about = "Judge in special mode")]
    Special(SpecialJudgeConfig),
    #[clap(version = "0.1.0", about = "Judge in interactive mode")]
    Interactive(InteractiveJudgeConfig),
    #[clap(version = "0.1.0", about = "Judge using config.yaml")]
    Prefab(PrefabJudgeConfig),
    #[clap(version = "0.1.0", about = "Debug mode")]
    Debug,
}

#[derive(Clap, Debug)]
struct NormalJudgeConfig {
    #[clap(about = "path of program to run")]
    exec: String,
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
    #[clap(about = "path of program to run")]
    exec: String,
    #[clap(short, about = "input file path")]
    input_file: String,
    #[clap(short, default_value = "1024", about = "memory limit(MB)")]
    memory_limit: u64,
    #[clap(short, default_value = "30000", about = "time limit(MS)")]
    time_limit: u64,
    #[clap(short, about = "checker program path")]
    checker: String,
}

#[derive(Clap, Debug)]
struct InteractiveJudgeConfig {
    #[clap(about = "interactor program path")]
    interactor: String,
    #[clap(about = "path of program to run")]
    exec: String,
    #[clap(short, about = "input file path")]
    input_file: String,
    #[clap(short, default_value = "1024", about = "memory limit(MB)")]
    memory_limit: u64,
    #[clap(short, default_value = "30000", about = "time limit(MS)")]
    time_limit: u64,
    
}

#[derive(Clap, Debug)]
struct PrefabJudgeConfig {
    #[clap(about = "problem config")]
    config: String,
    #[clap(about = "path of program to run")]
    exec: String,
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

            let judge_result = launch_normal_case_judge(
                &config.exec,
                &config.input_file,
                &config.answer_file,
                &LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
                &comparision_mode,
            )?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Special(config) => {
            let judge_result = launch_special_case_judge(
                &config.exec,
                &config.input_file,
                &config.checker,
                &LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
            )?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Prefab(config) => {
            let judge_result = ProblemConfig::from_file(&config.config)?.judge(&config.exec)?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Interactive(config) => {
            let judge_result = launch_interactive_case_judge(
                &config.exec,
                &config.input_file,
                &config.interactor,
                &LimitConfig {
                    time_limit: config.time_limit,
                    memory_limit: config.memory_limit,
                },
            );
            println!("{:#?}", judge_result);
        }
        SubCommand::Debug => {}
    }

    Ok(())
}
