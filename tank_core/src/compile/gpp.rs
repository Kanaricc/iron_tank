use std::{io::Read, process::{Command, Stdio}};
use std::fs::File;
use std::io::{Write};

use crate::{error::{Error,Result}, judge::get_path_of_tankcell, probe::ProcessProbe};

use super::{CompileResult, Compiler, CompilerEnvironmentStatus};

pub struct CompilerGPP {
    temp_dir: tempfile::TempDir,
    standard: GPPStandard,
    compiler_path:String,
}

#[derive(Debug,Clone)]
pub enum GPPStandard {
    CPP11,
    CPP17,
}

impl From<GPPStandard> for String{
    fn from(v: GPPStandard) -> Self {
        match v {
            GPPStandard::CPP11 => "-std=c++11".into(),
            GPPStandard::CPP17 => "-std=c++17".into(),
        }
    }
}

impl Compiler for CompilerGPP{
    fn check_environment() -> CompilerEnvironmentStatus {
        // TODO: check whether chosen standard is supported or not
        let path = which::which("g++");
        match path {
            Ok(path) => {
                let output = Command::new(&path)
                    .arg("--version")
                    .stdout(Stdio::piped())
                    .output()
                    .unwrap();
                let stdout = String::from_utf8(output.stdout).unwrap();
                let version = stdout.lines().next().unwrap().split(" ").last().unwrap();

                CompilerEnvironmentStatus::OK {
                    version: version.into(),
                    path: path.to_string_lossy().into(),
                }
            }
            Err(_) => CompilerEnvironmentStatus::Missing,
        }
    }

    fn compile(&self, src: String) ->CompileResult{
        let code_path = self.temp_dir.path().join("src.cpp");
        let exec_path = self.temp_dir.path().join("exec");

        {
            let mut file=File::create(&code_path).unwrap();
            file.write_all(&src.into_bytes()).unwrap();
            file.sync_all().unwrap();
        }

        // println!("{} {} {} {}",get_path_of_tankcell(),&self.compiler_path,"-p full",format!(
        //     "-- {} -o {} {}",
        //     code_path.to_str().unwrap(),
        //     exec_path.to_str().unwrap(),
        //     String::from(self.standard),
        // ));

        let command = Command::new(get_path_of_tankcell())
            .arg(&self.compiler_path)
            .arg("-p")
            .arg("full")
            .arg("--")
            .arg(code_path.to_str().unwrap())
            .arg("-o")
            .arg(exec_path.to_str().unwrap())
            .arg(String::from(self.standard.clone()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let probe=ProcessProbe::new(command.id()).unwrap();
        let probe=probe.watching();

        let mut stdout=String::new();
        command.stdout.unwrap().read_to_string(&mut stdout).unwrap();
        let mut stderr=String::new();
        command.stderr.unwrap().read_to_string(&mut stderr).unwrap();

        // TODO: handle limit
        println!("stat:{:?}\nout:{}\nerr:{}",probe,stdout,stderr);

        if probe.get_status()!=0{
            CompileResult::CompileError
        }else{
            CompileResult::OK
        }
    }
}

impl CompilerGPP {
    pub fn new() -> Result<Self> {
        // TODO: expose standard config
        match Self::check_environment(){
            CompilerEnvironmentStatus::OK { version: _, path } => {
                Ok(Self {
                    temp_dir: tempfile::TempDir::new().unwrap(),
                    standard: GPPStandard::CPP17,
                    compiler_path:path,
                })
            }
            CompilerEnvironmentStatus::Missing => {
                Err(Error::Environment("missing g++".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpp_environment() {
        let result = CompilerGPP::check_environment();
        match result{
            CompilerEnvironmentStatus::OK { version: _, path:_ } => {
                assert!(matches!(which::which("g++"),Ok(_)));
            }
            CompilerEnvironmentStatus::Missing => {
                assert!(matches!(which::which("g++"),Err(_)));
            }
        }
    }

    #[test]
    fn gpp_compile_error()->Result<()>{
        let src="#include <iostream> \n int main(){std::cout<<\"hi\"<<std::endl;}asd";
        let compiler=CompilerGPP::new()?;
        assert!(matches!(compiler.compile(src.into()),CompileResult::CompileError));

        Ok(())
    }

    #[test]
    fn gpp_compile_ok()->Result<()>{
        let src="#include <iostream> \n int main(){std::cout<<\"hi\"<<std::endl;}";
        let compiler=CompilerGPP::new()?;
        assert!(matches!(compiler.compile(src.into()),CompileResult::OK));

        Ok(())
    }
}