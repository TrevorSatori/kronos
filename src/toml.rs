use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use log::error;
use serde::de::DeserializeOwned;

#[derive(Debug)]
#[allow(dead_code)]
pub enum TomlFileError {
    NoPath(String),
    IoError(std::io::Error),
    TomlDeError(toml::de::Error),
    TomlSeError(toml::ser::Error),
}

impl From<std::io::Error> for TomlFileError {
    fn from(value: std::io::Error) -> Self {
        TomlFileError::IoError(value)
    }
}

impl From<toml::de::Error> for TomlFileError {
    fn from(value: toml::de::Error) -> Self {
        TomlFileError::TomlDeError(value)
    }
}

impl From<toml::ser::Error> for TomlFileError {
    fn from(value: toml::ser::Error) -> Self {
        TomlFileError::TomlSeError(value)
    }
}

fn get_config_dir_path() -> Result<PathBuf, TomlFileError> {
    home::home_dir().map(|path| path.as_path().join(".config/jolteon")).ok_or(TomlFileError::NoPath(".config/jolteon".to_string()))
}

fn get_config_file_path(file_name: &str) -> Result<PathBuf, TomlFileError> {
    get_config_dir_path().map(|path| path.as_path().join(file_name).with_extension("toml"))
}

fn create_dir() -> Result<(), TomlFileError> {
    let path = get_config_dir_path()?;
    Ok(create_dir_all(path)?)
}

pub fn read_toml_file<T>(file_name: &str) -> Result<T, TomlFileError>
where T: DeserializeOwned
{
    let path = get_config_file_path(file_name)?;
    let string = read_to_string(&path)?;
    Ok(toml::from_str(&string)?)
}

pub fn read_toml_file_or_default<T>(file_name: &str) -> T
where T: DeserializeOwned + Default
{
    read_toml_file(file_name).unwrap_or_else(|err| {
        error!(
            "Error '{file_name}' file. Will use default values. Error was: \n{:#?}",
            err
        );
        T::default()
    })
}

pub fn write_toml_file<T>(file_name: &str, file_contents: &T) -> Result<(), TomlFileError>
where T: serde::ser::Serialize
{
    create_dir()?;
    let path = get_config_file_path(file_name)?;
    let serialized = toml::to_string(file_contents)?;
    write(path, serialized)?;
    Ok(())
}
