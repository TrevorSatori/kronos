use std::collections::VecDeque;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cue::cue_line::CueLine;
use crate::cue::cue_line_node::CueLineNode;
use crate::cue::cue_sheet_item::CueSheetItem;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct CueSheet {
    cue_sheet_file_path: PathBuf,
    unknown: Vec<String>,
    comments: Vec<String>,
    performer: Option<String>,
    title: Option<String>,
    file: Option<CueFile>,
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct CueFile {
    name: String,
    tracks: Vec<Track>,
}

impl CueFile {
    fn new(name: String, mut c: Vec<CueSheetItem>) -> Self {
        let mut tracks = Vec::new();

        while let Some(t) = c.pop() {
            if let CueSheetItem::Track(track_index, track_properties) = t {
                tracks.push(Track::new(track_index, track_properties));
            }
        }

        tracks.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

        Self { name, tracks }
    }

    pub fn name(&self) -> String {
        let name = self.name.clone();
        let name_parts: Vec<&str> = name.split('"').filter(|s| !s.is_empty()).collect();
        name_parts[0].to_string()
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Track {
    index: String,
    // type: String (could be enum. always "audio" for now)
    title: String,
    start_time: String,
    performer: Option<String>,
}

impl Track {
    fn new(track_index: String, mut track_properties: Vec<CueSheetItem>) -> Self {
        let mut track = Self::default();
        track.index = track_index;

        while let Some(t) = track_properties.pop() {
            match t {
                CueSheetItem::Title(s) => { track.title = s }
                CueSheetItem::Performer(s) => { track.performer = Some(s) }
                CueSheetItem::Index(s) => { track.start_time = s }
                _ => {}
            }
        }

        track
    }

    pub fn index(&self) -> String {
        self.index.clone()
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn performer(&self) -> Option<String> {
        self.performer.clone()
    }

    pub fn start_time(&self) -> Duration {
        let start_time = self.start_time.clone();
        let start_time_parts: Vec<&str> = start_time.split_whitespace().filter(|s| !s.is_empty()).collect();
        let start_time_parts = start_time_parts[1].to_string();
        let mut time_parts: Vec<&str> = start_time_parts.split(':').collect();
        // MINUTES:SECONDS:FRAMES

        let _frames = match time_parts.pop() {
            Some(f) => str::parse(f).unwrap(),
            _ => 0,
        };

        let mut multiplier = 1u64;
        let mut seconds = 0u64;
        while let Some(t) = time_parts.pop() {
            let n: u64 = str::parse(t).unwrap();
            seconds += n * multiplier;
            multiplier *= 60;
        }

        let duration = Duration::from_secs(seconds);

        duration
    }
}

impl CueSheet {
    pub fn from_file(path: &Path) -> io::Result<CueSheet> {
        let cue_lines = CueLine::from_file(&path)?;
        let cue_nodes = CueLineNode::from_lines(VecDeque::from(cue_lines));
        let mut top_cue_items: Vec<CueSheetItem> =
            cue_nodes.iter().map(|n| CueSheetItem::from_cue_line_node(n)).collect();

        let mut sheet = CueSheet::default();
        sheet.cue_sheet_file_path = path.to_path_buf();

        while let Some(e) = top_cue_items.pop() {
            match e {
                CueSheetItem::Comment(s) => { sheet.comments.push(s) }
                CueSheetItem::Title(s) => { sheet.title = Some(s) }
                CueSheetItem::Performer(s) => { sheet.performer = Some(s) }
                CueSheetItem::File(s, c) => {
                    sheet.file = Some(CueFile::new(s, c));
                }
                _ => {}
            }
        }

        sheet.comments.sort();

        Ok(sheet)
    }

    pub fn cue_sheet_file_path(&self) -> PathBuf {
        self.cue_sheet_file_path.clone()
    }

    pub fn file(&self) -> Option<CueFile> {
        self.file.clone()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_sheet_from_file() {
        let path = Path::new("./src/cue/Tim Buckley - Happy Sad.cue");
        let cue = CueSheet::from_file(&path).unwrap();

        assert_eq!(cue.unknown.len(), 0);
        assert_eq!(cue.comments.len(), 4);

        assert_eq!(
            cue.comments,
            vec![
                "COMMENT \"ExactAudioCopy v0.99pb4\"",
                "DATE 1969",
                "DISCID 5B0A7D06",
                "GENRE Folk/Blues",
            ]
        );

        assert_eq!(cue.performer, Some("Tim Buckley".to_string()));

        let Some(file) = cue.file else { panic!() };

        assert_eq!(file.tracks.len(), 6, "{:#?}", file);

        assert_eq!(
            file.tracks[0],
            Track {
                index: "01 AUDIO".to_string(),
                title: "Strange Feelin'".to_string(),
                start_time: "01 00:00:00".to_string(),
                performer: Some("Tim Buckley".to_string())
            }
        );

        assert_eq!(
            file.tracks[1],
            Track {
                index: "02 AUDIO".to_string(),
                title: "Buzzin' Fly".to_string(),
                start_time: "01 07:41:25".to_string(),
                performer: Some("Tim Buckley".to_string())
            }
        );

        assert_eq!(
            file.tracks[5],
            Track {
                index: "06 AUDIO".to_string(),
                title: "Sing A Song For You".to_string(),
                performer: Some("Tim Buckley".to_string()),
                start_time: "01 42:06:30".to_string(),
            }
        );
    }
}
