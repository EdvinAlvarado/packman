use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader};
use regex::Regex;
use std::collections::HashMap;

enum Status {WrongParameters=1, DownloadFailed}

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
    pub fn run(&self) -> Result<(), i32>{
        match self.function {
            'S' => self.sync(),
            _   => self.man(),
        }
    }
    pub fn man(&self) -> Result<(), i32> {
        // TODO
        Ok(())
    }
    pub fn sync(&self) -> Result<(), i32> {
        for c in self.parameters.chars() {
            match c {
                'd' => {return self.download();},
                _   => {return Err(Status::WrongParameters as i32);},
            }
        }
        Ok(())
    }
    fn download(&self) -> Result<(), i32> {
        for package in self.packages.iter() {
            // println!("{}", package);
            let pkgbuild = pkgbuild_parser(format!("/home/edvin/mnt/lfs/sources/{}.pkgbuild", package));
            
            let re = Regex::new(format!("{}-.*$", package).as_str()).unwrap();
            let tarfile = re.captures(pkgbuild.get("source").unwrap())
                            .expect("tarfile regex failed")
                            .get(0)
                            .unwrap()
                            .as_str();
            
            if !Path::new(format!("/home/edvin/mnt/lfs/var/cache/packman/pkg/{}", tarfile).as_str()).exists() {
                let mut wget = Command::new("wget")
                    .arg(&pkgbuild["source"])
                    .arg("-P")
                    .arg("/home/edvin/mnt/lfs/var/cache/packman/pkg/")
                    .spawn()
                    .unwrap();
                wget.wait().expect("downloading file failed. Do you have internet? Is the URL correct?");
            }

            if pkgbuild["md5sums"] != "" {
                if checksum("md5sum", tarfile) != pkgbuild["md5sums"] {
                    println!("md5sum failed: {}", tarfile);
                    loop {
                        print!("Continue? [y/n]");
                        let mut answer = String::new();
                        let _stdin = std::io::stdin().read_line(&mut answer).unwrap();
                        if answer.contains("y") {
                            break;
                        } else if answer.contains("n"){
                            return Err(Status::DownloadFailed as i32);
                        }
                    }
                }
            } else if pkgbuild["sha256sums"] != "" {
                if checksum("sha256sum", tarfile) != pkgbuild["sha256sums"] {
                    println!("sha256sum failed: {}", tarfile);
                    loop {
                        print!("Continue? [y/n]");
                        let mut answer = String::new();
                        let _stdin = std::io::stdin().read_line(&mut answer).unwrap();
                        if answer.contains("y") {
                            break;
                        } else if answer.contains("n"){
                            return Err(Status::DownloadFailed as i32);
                        }
                    }
                }         
            }
        }
        Ok(())
    }

}

fn checksum(checksum: &'static str, tarfile: &str) -> String {
                let md5_output = Command::new(checksum)
                    .arg(format!("/home/edvin/mnt/lfs/var/cache/packman/pkg/{}", tarfile))
                    .output()
                    .expect(format!("{} error", checksum).as_str());
                return std::str::from_utf8(&md5_output.stdout).unwrap().split(' ').nth(0).unwrap().to_string();
                // println!("Compare:\n{}\n{}",md5,pkgbuild["md5sums"]);
}

fn pkgbuild_parser(filepath: String) -> HashMap<String, String> {
    let var_list = ["pkgname", "pkgver", "pkgrel", "pkgdesc", "arch", "depends", "license", "url", "source", "md5sums", "sha256sums"];
    let mut pkgbuild: HashMap<String, String> = HashMap::new();
    
    for var in var_list {
        let output = Command::new("bash")
            .arg("-c")
            .arg(format!("source {} && echo ${}", filepath, var))
            .output()
            .expect(format!("bash failed: {} - {}", filepath, var).as_str());
        
        let mut value = std::str::from_utf8(&output.stdout)
            .expect(format!("unknown bytes: {}", var).as_str())
            .to_string();
        value.pop();
        
        // println!("{} = {}", var, value);
        pkgbuild.insert(
            var.to_string(),
            value
        );
    }
    pkgbuild
}
