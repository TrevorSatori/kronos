use std::{fs::File, path::PathBuf};
use std::io::BufReader;
use rodio::{Sink, Decoder, OutputStream, source::Source};
use std::ffi::OsStr;
use std::thread;
// handles music streaming 


pub struct Music {
    pub sink: Sink,
    pub queue: Vec<PathBuf>,
    current_song: String,
}

impl Music {
    pub fn new() -> Music{
        
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).expect("Couldn't create sink");

        Music {    
            sink: sink,
            queue: Vec::new(),
            current_song: String::from("| Current Song |"),
        }
    }

    pub fn enqueu(&mut self, song: PathBuf){
        
        self.queue.push(song.clone());

        let file = BufReader::new(File::open("Lil Uzi Vert - Just Wanna Rock [1363459402].mp3").unwrap());
        // let source = Decoder::new(file).unwrap();
        // self.sink.append(source);
        // self.sink.sleep_until_end();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Add a dummy source of the sake of the example.
        // let source = SineWave::new(440.0).take_duration(Duration::from_secs_f32(0.25)).amplify(0.20);
        let source = Decoder::new(file).unwrap();
        sink.append(source);

        // The sound plays in a separate thread. This call will block the current thread until the sink
        // has finished playing all its queued sounds.
        sink.sleep_until_end();

    }

    pub fn pause(&self){
        self.sink.pause();
    }

    pub fn play(&self, file: BufReader<File>){

        thread::spawn(||{
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).expect("Couldn't create sink");
            // let file = BufReader::new(File::open(song).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
            sink.sleep_until_end();
            // self.sink.play();
        });
    }

    pub fn current_song(&self) -> String {
        self.current_song.clone()
    }
}