use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::io;
use regex::Regex;
use std::collections::HashMap;
extern crate reqwest;

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
    pub fn run(&self) -> Result<(), io::ErrorKind>{
        match self.function {
            'S' => self.sync(),
            _   => self.man(),
        }
    }
    pub fn man(&self) -> Result<(), io::ErrorKind> {
        // TODO
        Ok(())
    }
    pub fn sync(&self) -> Result<(), io::ErrorKind> {
        for c in self.parameters.chars() {
            match c {
                'd' => {return self.download();},
                _   => {return self.install();},
            }
        }
        Ok(())
    }
    fn install(&self) -> Result<(), io::ErrorKind> {
        // TODO
        Ok(())
    }
    fn download(&self) -> Result<(), io::ErrorKind> {
        for package in self.packages.iter() {
            // println!("{}", package);
            let pkgbuild = pkgbuild_parser(format!("/home/edvin/mnt/lfs/sources/{}.pkgbuild", package));
            
            let re = Regex::new(format!("{}-.*$", package).as_str()).unwrap();
            let tarfile = re.captures(pkgbuild.get("source").unwrap())
                            .expect("tarfile regex failed")
                            .get(0)
                            .unwrap()
                            .as_str();
            
            let tarpath_string = format!("/home/edvin/mnt/lfs/var/cache/packman/pkg/{}", tarfile);
            let tarpath = Path::new(tarpath_string.as_str());
            if !tarpath.exists() {
                let response = reqwest::blocking::get(&pkgbuild["source"]).expect("Download Failed").bytes().expect("download to bytes failed");
                let mut content = response.as_ref();
                let mut file = match File::create(&tarpath) {
                    Err(why) => panic!("Tar file could not be created: {}", why),
                    Ok(file) => file,
                };
                io::copy(&mut content, &mut file).expect("failed to download to tar path");
            }

            if pkgbuild["md5sums"] != "" {
                if checksum("md5sum", tarpath) != pkgbuild["md5sums"] {
                    println!("md5sum failed: {}", tarfile);
                    loop {
                        print!("Continue? [y/n]");
                        let mut answer = String::new();
                        let _stdin = io::stdin().read_line(&mut answer).unwrap();
                        if answer.contains("y") {
                            break;
                        } else if answer.contains("n"){
                            return Err(io::ErrorKind::InvalidData);
                        }
                    }
                }
            } else if pkgbuild["sha256sums"] != "" {
                if checksum("sha256sum", tarpath) != pkgbuild["sha256sums"] {
                    println!("sha256sum failed: {}", tarfile);
                    loop {
                        print!("Continue? [y/n]");
                        let mut answer = String::new();
                        let _stdin = std::io::stdin().read_line(&mut answer).unwrap();
                        if answer.contains("y") {
                            break;
                        } else if answer.contains("n"){
                            return Err(io::ErrorKind::InvalidData);
                        }
                    }
                }         
            }
        }
        Ok(())
    }

}

fn checksum(checksum: &'static str, tarfile: &Path) -> String {
                let md5_output = Command::new(checksum)
                    .arg(tarfile)
                    .output()
                    .expect(format!("{} error", checksum).as_str());
                // stdout = "<checksum> <file>"
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
