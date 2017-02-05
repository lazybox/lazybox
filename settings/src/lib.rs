#![feature(pub_restricted)]

extern crate yaml_rust;

mod value;
pub use self::value::*;

use std::{ops, fmt};
use std::path::Path;
use yaml_rust::Yaml;

pub struct Settings {
    values: ValueMap,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(defaults_path: P) -> Result<Self, Error> {
        Self::from_yaml(Self::read_yaml(defaults_path.as_ref()))
    }

    pub fn override_with<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        self.override_yaml(Self::read_yaml(path.as_ref()))
    }

    fn read_yaml(path: &Path) -> Yaml {
        use std::fs::File;
        use std::io::prelude::*;
        use yaml_rust::YamlLoader;

        let mut f = File::open(path).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        let docs = YamlLoader::load_from_str(&s).unwrap();
        docs.into_iter().next().unwrap_or(Yaml::Null)
    }

    pub fn from_yaml(yaml: Yaml) -> Result<Self, Error> {
        match yaml {
            Yaml::Hash(h) => Ok(Settings {
                values: try!(ValueMap::from(h)),
            }),
            Yaml::Null => Ok(Settings {
                values: ValueMap::empty(),
            }),
            _ => Err(Error::InvalidRoot),
        }
    }

    pub fn override_yaml(&mut self, yaml: Yaml) -> Result<(), Error> {
        match yaml {
            Yaml::Hash(h) => self.values.override_with(h),
            Yaml::Null => Ok(()),
            _ => Err(Error::InvalidRoot),
        }
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.values.fmt(f)
    }
}

impl ops::Deref for Settings {
    type Target = ValueMap;
    fn deref(&self) -> &ValueMap { &self.values }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The root must be an hash or null
    InvalidRoot,
    /// Keys must be strings
    InvalidKey,
    /// Trying to override a non-existent value
    InvalidOverride,
}
