extern crate serde;
extern crate toml;
extern crate reqwest;

use std::{collections::HashMap, error::Error};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use regex::Regex;
use std::process::Command;

#[derive(Deserialize)]
pub struct PackageBuild {
    package: Package,
    source: Source,
    depends: Option<Depends>,
    relations: Option<Relations>,
    others: Others
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    release: String,
    description: String,
    arch: Vec<String>,
    licenses: Vec<String>,
    url: String
}

#[derive(Deserialize)]
struct Source {
    sources: Vec<String>,
    md5sums: Option<Vec<String>>,
    sha256sums: Option<Vec<String>>
}

#[derive(Deserialize)]
struct Depends {
    depends: Option<HashMap<String, String>>,
    makedepends: Option<HashMap<String, String>>,
    optdepends: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct Relations {
    provides: Option<Vec<String>>,
    conflicts: Option<Vec<String>>,
    replaces: Option<Vec<String>>
}

#[derive(Deserialize)]
struct Others {
    backup: Option<Vec<PathBuf>>,
    install: Option<PathBuf>
}

#[derive(Debug)]
pub enum PackageError {
    ChecksumLenError,
    DifferentChecksumError,
    NoChecksumError,
    SourceLinkError,
}
impl std::error::Error for PackageError {}
impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PackageError::ChecksumLenError => write!(f, "checksums length error"),
            PackageError::DifferentChecksumError => write!(f, "different checksum error"),
            PackageError::NoChecksumError=> write!(f, "no checksum given"),
            PackageError::SourceLinkError => write!(f, "source doesn't link to tarfile")
        }
    }
}

static SOURCES_PATH: &str = "~/mnt/lfs/sources";
static PKGS_PATH: &str = "~/mnt/lfs/var/cache/packman/pkg";
static TEMP_PATH: &str = "~/mnt/lfs/tmp";

impl PackageBuild {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<PackageBuild, Box<dyn Error>> {
        let content = std::fs::read_to_string(file)?;
        let p: PackageBuild = toml::from_str(&content)?;

        // Does a check to confirm that the appropiate checksum are present
        match (&p.source.md5sums, &p.source.sha256sums) {
            (Some(md5sums), None) => if md5sums.len() != p.source.sources.len() {Err(Box::new(PackageError::ChecksumLenError))} else {Ok(p)},
            (None, Some(sha256sums)) => if sha256sums.len() != p.source.sources.len() {Err(Box::new(PackageError::ChecksumLenError))} else {Ok(p)},
            (Some(_), Some(_)) => Err(Box::new(PackageError::DifferentChecksumError)), 
            _ => Ok(p),  
        }
    }
    fn tarfile(&self, source: &str) -> Result<String, Box<dyn Error>> {
        let re = Regex::new(format!("{}-.*{}.+$", self.package.name, self.package.version).as_str())?;
        let tarfile = re.captures(source)
                        .ok_or(PackageError::SourceLinkError)?
                        .get(0)
                        .unwrap()
                        .as_str()
                        .to_string();
        Ok(tarfile.clone())
    }
    // Will download the files if they don't exist in /tmp or in cache.
    // and give back a list of the locations of the files.
    fn download_file<P: AsRef<Path>>(from: &str, to: P) {
        let mut resp = reqwest::blocking::get(from).expect("source download failed");
        let mut out = fs::File::create(to).expect("coudldn't create temp tarfile");
        io::copy(&mut resp, &mut out).expect("fail to download tarfile");
    }
    pub fn download(&self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut files: Vec<PathBuf> = Vec::new();
        for source in &self.source.sources {
            let mut pkg = PathBuf::new();
            pkg.push(PKGS_PATH);
            let mut tmp= PathBuf::new();
            tmp.push(TEMP_PATH);

            let filename = source.split("/").last().ok_or(PackageError::SourceLinkError)?;
            pkg.push(filename);
            tmp.push(filename);

            let file = match (pkg.exists(), tmp.exists()) {
                (false, true) => tmp,
                (false, false) => {
                    PackageBuild::download_file(source, &tmp);
                    tmp
                },
                (true, _) => pkg,
            };
            files.push(file);
        }
        Ok(files)
    }
    // will check if checksum is correct unless you pass "SKIP"
    fn checksum_file<P: AsRef<Path>>(checksum_type: &str, file: P, comparison: &str) -> Result<bool, Box<dyn Error>> {
        // [checksum] [file]
        if comparison == "SKIP" {return Ok(true);}
        let out = Command::new(checksum_type)
                        .arg(file.as_ref())
                        .output()?
                        .stdout;
        return Ok(std::str::from_utf8(&out)?.split(' ').nth(0).unwrap() == comparison);
    }
    // Will return a list of whether the file chcksum pass or not
    pub fn checksum(&self) -> Vec<Result<bool, Box<dyn Error>>> {
        let files = self.download().expect("download failed");
        let (check_type,checksums) = match (self.source.md5sums.as_ref(), self.source.sha256sums.as_ref()) {
            (Some(md5sums), None) => Ok(("md5sum", md5sums)),
            (None, Some(sha256sums)) => Ok(("sha256sum", sha256sums)),
            (Some(_), Some(_)) => Err(Box::new(PackageError::DifferentChecksumError)),
            (None, None) => Err(Box::new(PackageError::NoChecksumError)),
        }.expect("checksum error");

        let v = files.iter().zip(checksums.iter()).map(|t| PackageBuild::checksum_file(check_type, t.0, t.1)).collect();
        v
    }

}