use clap::Clap;
use std::{fs, io::Write, path::Path};
use std::process::{Command, Stdio};
#[derive(Clap)]
#[clap(
    version = "0.1.0",
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
}

#[derive(Clap,Debug)]
struct NormalJudge {
    exec: String,
    #[clap(short)]
    input_file: String,
    #[clap(short)]
    answer_file: String,
    #[clap(short)]
    memory_limit: String,
    #[clap(short)]
    time_limit: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Normal( config) => {
            let path=Path::new(&config.exec);
            let path=fs::canonicalize(path).unwrap().to_str().unwrap().to_string();

            let mut command = Command::new("./target/debug/tank_cell")
                .arg(path)
                .arg(format!("-m {}",config.memory_limit))
                .arg(format!("-t {}",config.time_limit))
                .arg("-p minimum")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let cin = command.stdin.as_mut().expect("failed to open stdin");
            cin.write_all("12".as_bytes())
                .expect("failed to write input");

            let output = command.wait_with_output().expect("Failed to read stdout");
            println!("{:#?}",output);
        }
    }

}
