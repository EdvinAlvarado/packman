use std::process::Command;
use std::fs::File;
use std::path::{Path, PathBuf};
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
            let pkg = Package::new(package.to_string());
            
            if !pkg.tarfile.exists() {
                let response = reqwest::blocking::get(&pkg.pkgbuild["source"]).expect("Download Failed").bytes().expect("download to bytes failed");
                let mut content = response.as_ref();
                let mut file = match File::create(&pkg.tarfile) {
                    Err(why) => panic!("Tar file could not be created: {}", why),
                    Ok(file) => file,
                };
                io::copy(&mut content, &mut file).expect("failed to download to tar path");
            }

            if pkg.pkgbuild["md5sums"] != "" {
                if pkg.checksum_match("md5sum") {
                    println!("md5sum failed: {}", pkg.tarfile.file_name().unwrap().to_str().unwrap());
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
            } else if pkg.pkgbuild["sha256sums"] != "" {
                if pkg.checksum_match("sha256sum") {
                    println!("sha256sum failed: {}", pkg.tarfile.file_name().unwrap().to_str().unwrap());
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
            }
        }
        Ok(())
    }

}



struct Package {
    pub pkgbuild: HashMap<String, String>,
    path_source: PathBuf,
    path_cache: PathBuf,
    pub tarfile: PathBuf,
}

impl Package {
    pub fn new(package: String) -> Package {
        let mut sources = PathBuf::new();
        sources.push("~/mnt/lfs/sources");
        let mut cache = PathBuf::new();
        cache.push("~/mnt/lfs/var/cache/packman/pkg");
        let re = Regex::new(format!("{}-.*$", package).as_str()).unwrap();
        let pkgbuild_hash = Package::pkgbuild_parser(sources.join(package));
        let url = pkgbuild_hash.get("source").unwrap().to_string();
        let tarfile_name = re.captures(url.as_str())
                        .expect("tarfile regex failed")
                        .get(0)
                        .unwrap()
                        .as_str()
                        .to_string();

        Package {
            pkgbuild: pkgbuild_hash,
            path_source: sources,
            path_cache: cache.clone(),
            tarfile: cache.join(tarfile_name),
        }
    }
    fn pkgbuild_parser(filepath: PathBuf) -> HashMap<String, String> {
        let var_list = ["pkgname", "pkgver", "pkgrel", "pkgdesc", "arch", "depends", "license", "url", "source", "md5sum", "sha256sum"];
        let mut pkgbuild: HashMap<String, String> = HashMap::new();
        
        for var in var_list {
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("source {} && echo ${}", filepath.display(), var))
                .output()
                .expect(format!("bash failed: {} - {}", filepath.display(), var).as_str());
            
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
    pub fn checksum(&self, checksum: &'static str) -> String {
        let md5_output = Command::new(checksum)
            .arg(self.tarfile.clone())
            .output()
            .expect(format!("{} error", checksum).as_str());
        // stdout = "<checksum> <file>"
        return std::str::from_utf8(&md5_output.stdout).unwrap().split(' ').nth(0).unwrap().to_string();
                // println!("Compare:\n{}\n{}",md5,pkgbuild["md5sums"]);
    }
    pub fn checksum_match(&self, checksum_type: &'static str) -> bool {
        self.checksum(checksum_type) == self.pkgbuild[checksum_type]
    }
}



