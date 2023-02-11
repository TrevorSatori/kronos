use std::{path::{Path, PathBuf}, env, ffi::OsStr};
use glob::{glob_with, MatchOptions};
use lofty::{Probe, TaggedFileExt, Accessor};


// converts queue items to what's displayed for user
pub fn audio_display(path: &PathBuf) -> String {

    let path = Path::new(&path);
    let tagged_file = Probe::open(path)
    .expect("ERROR: Bad path provided!")
    .read()
    .expect("ERROR: Failed to read file!");

    let ptag = tagged_file.primary_tag().unwrap();
    let artist = ptag.artist().as_deref().unwrap().to_string();
    let title = tagged_file.primary_tag().unwrap().title().unwrap();
    return artist + " - " + &title.clone();
}

// scans folder for valid files, returns matches
pub fn scan_folder() -> Vec<String>{

    let mut items = Vec::new();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    
    for e in glob_with("./*", options).expect("Failed to read glob pattern") {
        match e {
            Ok(item) => {
                
                let current_dir = env::current_dir().unwrap();
                let join = Path::join(&current_dir, Path::new(&item));
                let ext = Path::new(&item).extension().and_then(OsStr::to_str);       
            
                // if folder  (Hide Private) enter, else play song
                if (join.is_dir() && !join.file_name().unwrap().to_str().unwrap().contains(".") ) || (ext.is_some() && 
                (item.extension().unwrap() == "mp3" || 
                item.extension().unwrap() == "mp4" || 
                item.extension().unwrap() == "m4a" || 
                item.extension().unwrap() == "wav" || 
                item.extension().unwrap() == "flac" )){
                    items.push(item.to_str().unwrap().to_owned());
                }         
            },
            Err(_) => (),
        }
    }
    return items;
}