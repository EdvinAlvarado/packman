extern crate reqwest;
use crate::package::PackageBuild;

use std::fs::File;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::io;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;


#[derive(Debug)]
pub enum ProgramError {
    WrongParameters,
    DownloadFailed,
}
impl std::error::Error for ProgramError {}
impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProgramError::DownloadFailed => write!(f, "download failed"),
            ProgramError::WrongParameters=> write!(f, "wrong parameters"),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub function: char,
    pub parameters: String,
    pub packages: Vec<String>
}

impl Program {
    pub fn new() -> Program {
        Program {
            function: ' ',
            parameters: String::new(),
            packages: Vec::new(),
        }
    }
    pub fn run(&self) -> Result<(), Box<dyn Error>>{
        match self.function {
            'S' => self.sync(),
            _   => self.man(),
        }
    }
    pub fn man(&self) -> Result<(), Box<dyn Error>> {
        // TODO
        Ok(())
    }
    pub fn sync(&self) -> Result<(), Box<dyn Error>> {
        for c in self.parameters.chars() {
            match c {
                'd' => {return self.download();},
                _   => {return self.install();},
            }
        }
        Ok(())
    }
    fn install(&self) -> Result<(), Box<dyn Error>> {
        // TODO
        Ok(())
    }
    fn download(&self) -> Result<(), Box<dyn Error>> {
        for package in self.packages.iter() {
            // println!("{}", package);
            let pkg = PackageBuild::new(package.to_string())?;
            
        }
        Ok(())
    }

}
