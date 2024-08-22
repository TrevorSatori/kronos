use rodio::Sink;

pub trait ExtendedSink {
    fn toggle(&self);
    fn change_volume(&self, amount: f32);
}

impl ExtendedSink for Sink {
    fn toggle(&self) {
        if self.is_paused() {
            self.play()
        } else {
            self.pause()
        }
    }

    fn change_volume(&self, amount: f32) {
        let mut volume = self.volume() + amount;
        if volume < 0. {
            volume = 0.;
        } else if volume > 1. {
            volume = 1.;
        }
        self.set_volume(volume)
    }
}
