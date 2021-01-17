use clap::Clap;
use iron_tank::{error::{Error, Result}, judge::{Judge, NormalJudge}};
use iron_tank::{
    compare::{self, CompareMode},
};
use std::{fs, path::Path};
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
struct DebugJudge {}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Normal(config) => {
            let path = Path::new(&config.exec);
            let input_file_path = Path::new(&config.input_file);
            let answer_file_path = Path::new(&config.answer_file);

            if !path.exists() || !input_file_path.exists() || !answer_file_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "code, input or answer not found",
                )
                .into());
            }

            let input=fs::read_to_string(input_file_path)?;
            let answer=fs::read_to_string(answer_file_path)?;

            let comparation:Box<dyn CompareMode>=match config.compare_method.as_str() {
                "full"=>Box::new(compare::GlobalCompare{}),
                "line"=>Box::new(compare::LineCompare{}),
                "value"=>Box::new(compare::ValueCompare{}),
                _=>{
                    Err(Error::Argument("comparation mode not found".into()))?
                }
            };

            let judge=NormalJudge::new(config.exec, input, answer, config.memory_limit, config.time_limit, comparation);
            let judge_result=judge.judge()?;
            println!("{:#?}", judge_result);
        }
        SubCommand::Debug => {}
    }

    Ok(())
}
