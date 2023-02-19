use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use glob::{glob_with, MatchOptions};
use lofty::{Accessor, Probe, TaggedFileExt};

// converts queue items to what's displayed for user
pub fn audio_display(path: &PathBuf) -> String {
    let path = Path::new(&path);
    let tagged_file = Probe::open(path)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

    let ptag = tagged_file.primary_tag().unwrap();
    let artist = ptag.artist();

    // if filename
    if let Some(i) = tagged_file.primary_tag().unwrap().title() {
        // if artist data
        if let Some(j) = artist {
            format!("{artist} - {title}", artist = j, title = i)
        } else {
            i.into()
        }
    } else {
        path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

// scans folder for valid files, returns matches
pub fn scan_folder() -> Vec<String> {
    let mut items = Vec::new();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    for item in glob_with("./*", options)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&item));
        let ext = Path::new(&item).extension().and_then(OsStr::to_str);

        // if folder  (Hide Private) enter, else play song
        if (join.is_dir() && !join.file_name().unwrap().to_str().unwrap().contains('.'))
            || (ext.is_some()
                && (item.extension().unwrap() == "mp3"
                    || item.extension().unwrap() == "mp4"
                    || item.extension().unwrap() == "m4a"
                    || item.extension().unwrap() == "wav"
                    || item.extension().unwrap() == "flac"
                    || item.extension().unwrap() == "ogg"
                    || item.extension().unwrap() == "aac"))
        {
            items.push(item.to_str().unwrap().to_owned());
        }
    }
    items
}

// scans folder for valid files, returns matches
// need to set current dir
pub fn bulk_add(selected: &PathBuf) -> Vec<PathBuf> {
    let mut items = Vec::new();
    env::set_current_dir(selected).unwrap();

    for item in glob::glob("./*")
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let current_dir = env::current_dir().unwrap();
        let join = Path::join(&current_dir, Path::new(&item));
        let ext = Path::new(&item).extension().and_then(OsStr::to_str);
        if ext.is_some()
            && (item.extension().unwrap() == "mp3"
                || item.extension().unwrap() == "mp4"
                || item.extension().unwrap() == "m4a"
                || item.extension().unwrap() == "wav"
                || item.extension().unwrap() == "flac"
                || item.extension().unwrap() == "ogg"
                || item.extension().unwrap() == "aac")
        {
            items.push(join);
        }
    }
    env::set_current_dir("../").unwrap();
    items
}
