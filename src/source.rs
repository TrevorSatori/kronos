use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{
    Decoder,
    Source as RodioSource,
    source::{Amplify, Pausable, PeriodicAccess, SamplesConverter, Skippable, Speed, Stoppable, TrackPosition, SeekError},
};

type FullRodioSource = Stoppable<Skippable<Amplify<Pausable<TrackPosition<Speed<Decoder<BufReader<File>>>>>>>>;
type PeriodicRodioSource<F> = SamplesConverter<PeriodicAccess<FullRodioSource, F>, f32>;

pub struct Controls<'a> {
    src: &'a mut FullRodioSource,
    shared_pos: &'a Arc<Mutex<Duration>>,
}

impl Controls<'_> {

    #[inline]
    pub fn stop(&mut self) {
        self.src.stop();
        self.set_pos(Duration::ZERO);
    }

    #[inline]
    pub fn skip(&mut self) {
        self.src.inner_mut().skip();
    }

    #[inline]
    pub fn pos(&self) -> Duration {
        self.src.inner().inner().inner().inner().get_pos()
    }

    #[inline]
    pub fn set_pos(&self, pos: Duration) {
        *self.shared_pos.lock().unwrap() = pos;
    }

    #[inline]
    pub fn refresh_pos(&self) {
        self.set_pos(self.pos());
    }

    #[inline]
    pub fn set_volume(&mut self, factor: f32) {
        self.src.inner_mut().inner_mut().set_factor(factor)
    }

    #[inline]
    pub fn set_paused(&mut self, paused: bool) {
        self.src.inner_mut().inner_mut().inner_mut().set_paused(paused)
    }

    #[inline]
    pub fn seek(&mut self, position: Duration) -> Result<(), SeekError> {
        self.src.try_seek(position)
    }
}

pub struct Source<F> {
    input: PeriodicRodioSource<F>,
    path: PathBuf,
    on_playback_end: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl Source<()> {
    pub fn from_file(
        path: PathBuf,
        mut periodic_access: impl FnMut(&mut Controls) + Send,
        shared_pos: Arc<Mutex<Duration>>,
        on_playback_end: impl FnOnce() + Send + 'static,
    ) -> Source<Box<impl FnMut(&mut FullRodioSource) + Send>>
    {
        let periodic_access_inner = {
            Box::new(move |src: &mut FullRodioSource| {
                let mut controls = Controls { src, shared_pos: &shared_pos };
                controls.refresh_pos();
                periodic_access(&mut controls);
            })
        };

        let file = BufReader::new(File::open(path.clone()).unwrap());
        let source = Decoder::new(file).unwrap();
        let input = source
            .speed(1.0)
            .track_position()
            .pausable(false)
            .amplify(1.0)
            .skippable()
            .stoppable()
            .periodic_access(Duration::from_millis(5), periodic_access_inner)
            .convert_samples();

        Source {
            input,
            path,
            on_playback_end: Some(Box::new(on_playback_end)),
        }
    }
}

impl<F: FnMut(&mut FullRodioSource) + Send> Source<F>
where
    F: FnMut(&mut FullRodioSource) + Send,
{

    #[inline]
    pub fn _inner_mut(&mut self) -> &mut PeriodicRodioSource<F> {
        &mut self.input
    }

    pub fn seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let i = self.input.inner_mut().inner_mut().inner_mut();
        i.try_seek(pos)
    }

    pub fn _skip(&mut self) -> () {
        let i = self.input.inner_mut().inner_mut().inner_mut();
        i.skip()
    }
}

impl<F> Iterator for Source<F>
where
    F: FnMut(&mut FullRodioSource),
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let n = self.input.next();

        if n.is_none() {
            if let Some(cb) = self.on_playback_end.take() {
                cb();
            }
        }

        n
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<F> RodioSource for Source<F>
where
    F: FnMut(&mut FullRodioSource),
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.input.try_seek(pos)
    }
}

impl<F> Drop for Source<F> {
    fn drop(&mut self) {
        log::trace!("Source.drop()");
    }
}
