use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use log::error;

use crate::extensions::string::StringExtensions;

#[derive(Eq, PartialEq)]
pub struct CueLine {
    pub indentation: usize,
    pub key: String,
    pub value: String,
}

impl Display for CueLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key, self.value)
    }
}

impl Debug for CueLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key, self.value)
    }
}

impl CueLine {
    pub fn from_reader<B>(lines: io::Lines<B>) -> Vec<CueLine>
    where B: BufRead, {
        let mut cue_lines = Vec::new();

        for line in lines.flatten() {
            let indentation = line.count_leading_whitespace();
            let key_value = line.trim_leading_whitespace();

            let Some((key, value)) = key_value.split_once(char::is_whitespace) else {
                error!("lines should be key value {:?}", line);
                continue;
            };

            cue_lines.push(Self {
                indentation,
                key: key.to_string(),
                value: value.to_string(),
            });

        }

        cue_lines
    }

    pub fn from_file(path: &Path) -> io::Result<Vec<CueLine>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file).lines();
        Ok(Self::from_reader(reader))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_lines_from_file() {
        let path = Path::new("./src/cue/Tim Buckley - Happy Sad.cue");
        let cue_lines = CueLine::from_file(&path).unwrap();

        assert_eq!(cue_lines.len(), 31, "{:#?}", cue_lines);

        assert_eq!(cue_lines[0], CueLine {
            indentation: 0,
            key: "REM".to_string(),
            value: "GENRE Folk/Blues".to_string(),
        });

        assert_eq!(cue_lines[4], CueLine {
            indentation: 0,
            key: "PERFORMER".to_string(),
            value: "\"Tim Buckley\"".to_string(),
        });

        assert_eq!(cue_lines[5], CueLine {
            indentation: 0,
            key: "TITLE".to_string(),
            value: "\"Happy Sad\"".to_string(),
        });
    }

}
