#![feature(pub_restricted)]

extern crate yaml_rust;

mod value;
pub use self::value::*;

use std::fmt;
use yaml_rust::Yaml;

pub struct Settings {
    values: ValueMap,
}

impl Settings {
    pub fn new(defaults_path: &str) -> Result<Self, Error> {
        Self::from_yaml(Self::read_yaml(defaults_path))
    }

    pub fn override_with(&mut self, path: &str) -> Result<(), Error> {
        self.override_yaml(Self::read_yaml(path))
    }

    fn read_yaml(path: &str) -> Yaml {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The root must be an hash or null
    InvalidRoot,
    /// Keys must be strings
    InvalidKey,
    /// Trying to override a non-existent value
    NoneOverride,
    /// Trying to override a value with a different type
    OverrideMismatch,
}
