use clap::Clap;
use iron_tank::error::{Error, Result};
use iron_tank::{
    compare::{self, CompareMode},
    probe::ProcessProbe,
    JudgeResult, JudgeStatus,
};
use std::process::{Command, Stdio};
use std::{fs, io::Read, io::Write, path::Path};
#[derive(Clap)]
#[clap(
    version = "0.1.1",
    author = "Kanari <iovo7c@gmail.com>",
    about = "A fast and reliable judge container wrtten in Rust."
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "0.1.0", about = "Normal mode")]
    Normal(NormalJudge),
    #[clap(version = "0.1.0", about = "Debug mode")]
    Debug,
}

#[derive(Clap, Debug)]
struct NormalJudge {
    exec: String,
    #[clap(short)]
    input_file: String,
    #[clap(short)]
    answer_file: String,
    #[clap(short)]
    memory_limit: u64,
    #[clap(short)]
    time_limit: u64,
    #[clap(short)]
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
                    "stage1: code, input or answer not found",
                )
                .into());
            }

            let path = fs::canonicalize(path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let mut command = Command::new("./target/debug/tank_cell")
                .arg(path)
                .arg(format!("-m {}", config.memory_limit))
                .arg(format!("-t {}", config.time_limit))
                .arg("-p minimum")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let pid = command.id();
            let probe =
                ProcessProbe::new(pid).expect("stage2: failed to insert probe into process");

            let cin = command
                .stdin
                .as_mut()
                .expect("stage2: failed to open stdin");
            cin.write_all(&fs::read(config.input_file).unwrap().into_boxed_slice())
                .expect("stage2: failed to write input");

            let cout = command
                .stdout
                .as_mut()
                .expect("stage2: failed to open stdout");

            let probe_res = probe.watching();
            let mut output = String::new();
            cout.read_to_string(&mut output)
                .expect("stage3: failed to read stdout");

            // check result
            let mut judge_status = if probe_res.get_time_usage() >= config.time_limit {
                JudgeStatus::TimeLimitExceeded
            } else if probe_res.get_peak_memory() >= config.memory_limit * 1024 {
                JudgeStatus::MemoryLimitExceeded
            } else if probe_res.get_status() != 0 {
                JudgeStatus::RuntimeError
            } else {
                JudgeStatus::Uncertain
            };

            let answer = fs::read_to_string(answer_file_path).unwrap();

            if let JudgeStatus::Uncertain = judge_status {
                judge_status = match config.compare_method.as_str() {
                    "full" => compare::GlobalCompare::compare(&answer, &output).into(),
                    "line" => compare::LineCompare::compare(&answer, &output).into(),
                    "value" => compare::ValueCompare::compare(&answer, &output).into(),
                    _ => {
                        return Err(Error::Argument(format!(
                            "stage3: no compare method named `{}`",
                            config.compare_method
                        )));
                    }
                }
            }

            let judge_result = JudgeResult {
                status: judge_status,
                time: probe_res.get_time_usage(),
                memory: probe_res.get_peak_memory(),
            };

            println!("{:#?}", judge_result);
        }
        SubCommand::Debug => {}
    }

    Ok(())
}
