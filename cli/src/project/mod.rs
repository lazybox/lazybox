mod runner;

pub use self::runner::Runner;
use std::path::{Path, PathBuf};

pub enum TargetType<'a> {
    Release(&'a str),
    Debug(&'a str),
}

impl<'a> Into<PathBuf> for TargetType<'a> {
    fn into(self) -> PathBuf {
        match self {
            TargetType::Release(name) => name.into(),
            TargetType::Debug(name) => name.into(),
        }
    }
}

pub struct Project {
    game_library: PathBuf,
}

impl Project {
    pub fn new(name: &str, target_type: TargetType) -> Self {
        let mut target: PathBuf = "./target".into();
        target.push::<PathBuf>(target_type.into());
        target.push(name);

        Project {
            game_library: target 
        }
    } 
}