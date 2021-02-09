use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

use super::{CompileResult, CompiledProgram, Compiler, CompilerDescriptor, CompilerEnvironmentStatus};
use crate::error::{Error, Result};

pub struct CompilerPython {
    temp_dir: tempfile::TempDir,
    compiler_path: String,
}

impl CompilerPython {
    pub fn new() -> Result<Self> {
        match Self::check_environment() {
            CompilerEnvironmentStatus::OK { version: _, path } => Ok(Self {
                temp_dir: tempfile::TempDir::new().unwrap(),
                compiler_path: path,
            }),
            CompilerEnvironmentStatus::Missing => Err(Error::Environment("missing python".into())),
        }
    }
}

impl CompilerDescriptor for CompilerPython{
    fn support_sufix()->Vec<&'static str> {
        vec!["py"]
    }

    fn check_environment() -> super::CompilerEnvironmentStatus {
        let path = which::which("python");
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
                    path: path.to_string_lossy().to_string(),
                }
            }
            Err(_) => CompilerEnvironmentStatus::Missing,
        }
    }
}

impl Compiler for CompilerPython {
    fn compile(&self, src: String) -> super::CompileResult {
        let code_path = self.temp_dir.path().join("src.cpp");

        {
            let mut file = File::create(&code_path).unwrap();
            file.write_all(&src.into_bytes()).unwrap();
            file.sync_all().unwrap();
        }

        CompileResult::OK(CompiledProgram::new_with_args(
            self.compiler_path.clone(),
            vec![code_path.to_string_lossy().to_string()],
        ))
    }
}
