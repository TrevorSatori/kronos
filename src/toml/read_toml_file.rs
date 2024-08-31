use std::fs;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum TomlFileError {
    NoPath,
    IoError(std::io::Error),
    TomlError(toml::de::Error),
}

impl From<std::io::Error> for TomlFileError {
    fn from(value: std::io::Error) -> Self {
        TomlFileError::IoError(value)
    }
}

impl From<toml::de::Error> for TomlFileError {
    fn from(value: toml::de::Error) -> Self {
        TomlFileError::TomlError(value)
    }
}

pub fn read_toml_file<T>(file_name: &str) -> Result<T, TomlFileError>
where T: DeserializeOwned
{
    let Some(path) = home::home_dir() else {
        return Err(TomlFileError::NoPath);
    };
    let mut path = path.as_path().join(".config/jolteon").join(file_name);
    path.set_extension("toml");
    let config_string = fs::read_to_string(&path)?;
    Ok(toml::from_str(&config_string)?)
}
