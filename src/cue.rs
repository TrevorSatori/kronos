use std::time::Duration;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub struct CueSheet {
    unknown: Vec<String>,
    comments: Vec<String>,
    performer: Option<String>,
    title: Option<String>,
    file: Option<String>,
    tracks: Vec<Track>,
}

#[derive(Debug)]
pub struct Track {
    index: String,
    // type: String (could be enum. always "audio" for now)
    title: String,
    start_time: Duration,
    performer: Option<String>,
}

#[derive(Debug)]
pub struct CueLine {
    indentation: usize,
    key: String,
    value: String,
}

impl CueLine {
    pub fn from_file(path: &Path) -> io::Result<Vec<CueLine>> {
        let mut cue_lines = Vec::new();

        for line in read_lines(path)?.flatten() {
            let indentation = line.count_leading_whitespace();
            let key_value = line.trim_leading_whitespace();

            let Some((key, value)) = key_value.split_once(char::is_whitespace) else {
                eprintln!("lines should be key value {:?}", line);
                continue;
            };

            cue_lines.push(Self {
                indentation,
                key: key.to_string(),
                value: value.strip_quotes().to_string(),
            });

        }

        Ok(cue_lines)
    }
}

impl CueSheet {
    pub fn from_file(path: &Path) -> io::Result<CueSheet> {
        let cue_lines = CueLine::from_file(&path)?;

        let mut performer: Option<String> = None;
        let mut title: Option<String> = None;
        let mut file: Option<String> = None;
        let mut comments = Vec::new();
        let mut tracks = Vec::new();
        let mut unknown = Vec::new();

        let mut is_under_track = false;
        let mut track_index: Option<String> = None;
        let mut track_title: Option<String> = None;
        let mut track_performer: Option<String> = None;

        for line in cue_lines {
            match line.key.as_ref() {
                "REM" => {
                    comments.push(line.value);
                }
                "PERFORMER" => {
                    if is_under_track {
                        track_performer = Some(line.value)
                    } else {
                        performer = Some(line.value);
                    }
                }
                "TITLE" => {
                    if is_under_track {
                        track_title = Some(line.value)
                    } else {
                        title = Some(line.value);
                    }
                }
                "FILE" => {
                    file = Some(line.value);
                }
                "TRACK" => {
                    if is_under_track {
                        tracks.push(Track {
                            index: track_index.unwrap(),
                            title: track_title.clone().unwrap_or("wtf".to_string()),
                            performer: track_performer.clone(),
                            start_time: Default::default(),
                        });
                    }
                    is_under_track = true;
                    track_index = Some(line.value);
                }
                "INDEX" => {
                    track_index = Some(line.value);
                }
                _ => {
                    unknown.push(line.key);
                }
            }
        }

        Ok(Self {
            unknown,
            performer,
            title,
            comments,
            file,
            tracks,
        })
    }
}

fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

pub trait StringExtras {
    fn count_leading_whitespace(&self) -> usize;
    fn trim_leading_whitespace(&self) -> &str;
}

impl StringExtras for String {
    fn count_leading_whitespace(&self) -> usize {
        let mut indentation = 0;

        for char in self.chars() {
            if char.is_whitespace() {
                indentation += 1;
            } else {
                break;
            }
        }

        indentation
    }

    fn trim_leading_whitespace(&self) -> &str {
        self.strip_prefix(char::is_whitespace).unwrap_or(self).trim_start()
    }
}

pub trait StrExtras {
    fn strip_quotes(&self) -> &str;
}

impl StrExtras for str {
    fn strip_quotes(&self) -> &str {
        self.trim_matches('"')
    }
}
