use std::process::Command;

use crate::compare;


pub struct Cell{
    command:Command,
}

impl Cell {
    pub fn new_executable(path:String)->Self{
        Self{
            command:Command::new(path)
        }
    }

    pub fn new_interpretive(interpretor:String,path:String)->Self{
        Self{
            command:Command::new(interpretor)
        }
    }
}
