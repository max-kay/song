use std::rc::Rc;

pub mod auto;
pub mod consts;
pub mod effects;
pub mod instruments;
pub mod io;
pub mod time;
pub mod tracks;
pub mod utils;
pub mod wave;

pub struct Song<W: wave::Wave> {
    name: String,
    tracks: Vec<tracks::Track<W>>,
    time_keeper: Rc<time::TimeKeeper>,
}

impl<W: 'static + wave::Wave> Song<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tracks: Vec::new(),
            time_keeper: Rc::new(time::TimeKeeper::default()),
        }
    }

    pub fn add_midi_track(&mut self, track: tracks::MidiTrack<W>) {
        self.tracks.push(tracks::Track::Midi(track))
    }

    pub fn get_wave(&self) -> W {
        let mut wave = W::new();
        for track in &self.tracks {
            wave.add_consuming(track.play(), 0);
        }
        wave
    }
}
