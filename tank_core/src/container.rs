use std::process::Command;

#[allow(dead_code)]
pub struct Cell {
    command: Command,
}

impl Cell {
    #[allow(dead_code)]
    pub fn new_executable(path: String) -> Self {
        Self {
            command: Command::new(path),
        }
    }

    #[allow(dead_code)]
    pub fn new_interpretive(interpretor: String, _path: String) -> Self {
        Self {
            command: Command::new(interpretor),
        }
    }
}
