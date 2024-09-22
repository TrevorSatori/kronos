use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

use rodio::{
    Decoder,
    Source,
    source::{Amplify, Pausable, PeriodicAccess, SamplesConverter, Skippable, Speed, Stoppable, TrackPosition},
};

pub type FullSource = Stoppable<Skippable<Amplify<Pausable<TrackPosition<Speed<Decoder<BufReader<File>>>>>>>>;

pub fn create_source<F>(source: Decoder<BufReader<File>>, periodic_access: F) -> SamplesConverter<PeriodicAccess<FullSource, F>, f32>
where F: FnMut(&mut FullSource)
{
    source
        .speed(1.0)
        .track_position()
        .pausable(false)
        .amplify(1.0)
        .skippable()
        .stoppable()
        .periodic_access(Duration::from_millis(5), periodic_access)
        .convert_samples()
}

pub fn create_source_from_file<F>(path: PathBuf, periodic_access: F) -> SamplesConverter<PeriodicAccess<FullSource, F>, f32>
where F: FnMut(&mut FullSource)
{
    let file = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(file).unwrap();
    create_source(source, periodic_access)
}
