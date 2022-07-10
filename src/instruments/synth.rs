use crate::utils::envelope::Envelope;
use crate::utils::oscs;
use super::MidiInstrument;
use crate::auto;
use crate::auto::TimeFunction;
use crate::io;
use crate::time;
use crate::tracks::midi;
use crate::wave::Wave;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

pub struct SynthAutomation {
    lfo1: auto::Lfo,
    lfo2: auto::Lfo,
    pitchbend: auto::PointDefined,
    modulation_wheel: auto::PointDefined,
    main_envelope: Envelope,
    alt_envelope: Envelope,
    time_keeper: Rc<time::TimeKeeper>,
}

impl SynthAutomation {
    fn empty(time_keeper: Rc<time::TimeKeeper>) -> Self {
        let lfo1 = auto::Lfo::default_lfo();
        let lfo2 = auto::Lfo::default_lfo();
        let pitchbend = auto::PointDefined::one_point(0.5, Rc::clone(&time_keeper));
        let modulation_wheel = auto::PointDefined::one_point(0.5, Rc::clone(&time_keeper));
        Self {
            lfo1,
            lfo2,
            pitchbend,
            modulation_wheel,
            time_keeper,
            main_envelope: Envelope::default(),
            alt_envelope: Envelope::default(),
        }
    }

    pub fn get_main_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.main_envelope.get_envelope(sus_samples)
    }

    pub fn get_pitch_vec(&self, onset: time::TimeStamp, samples: usize) -> Vec<f64> {
        self.pitchbend.get_vec(onset, samples)
    }
}

pub struct Connections {
    connections: Vec<auto::Control>,
}

pub struct Synthesizer<W: Wave> {
    phantom: PhantomData<W>,
    name: String,
    oscillators: Vec<Box<dyn oscs::Oscillator<W>>>,
    local_automation: SynthAutomation,
    connections: Connections,
    pitch_wheel_range: f64, // in cents
    time_keeper: Rc<time::TimeKeeper>,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(
        name: String,
        oscillators: Vec<Box<dyn oscs::Oscillator<W>>>,
        local_automation: SynthAutomation,
        connections: Connections,
        time_keeper: Rc<time::TimeKeeper>,
    ) -> Self {
        Self {
            phantom: PhantomData,
            name,
            oscillators,
            local_automation,
            connections,
            pitch_wheel_range: 2.0,
            time_keeper,
        }
    }

    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.local_automation.get_main_envelope(sus_samples)
    }
}

impl<W: Wave> Synthesizer<W> {
    fn play_freq(
        &self,
        note_on: time::TimeStamp,
        note_off: time::TimeStamp,
        freq: f64,
        velocity: f64,
    ) -> W {
        let sus_samples = self.time_keeper.duration_to_samples(note_on, note_off);
        let envelope = self.get_envelope(sus_samples);
        let freq = self
            .local_automation
            .get_pitch_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf((x * 2.0 - 1.0) * self.pitch_wheel_range / 1200.0))
            .collect();
        let freq = auto::ValOrVec::Vec(freq);
        let modulation = auto::ValOrVec::Val(0.5);
        let mut wave = W::zeros(envelope.len());
        for osc in &self.oscillators {
            wave.add_consuming(osc.wave(&freq, &modulation, envelope.len()), 0);
        }
        wave.scale_by_vec(envelope);
        wave
    }
}

impl<W: Wave> MidiInstrument<W> for Synthesizer<W> {
    fn play_note(&self, note: midi::Note) -> W {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &Vec<midi::Note>) -> W {
        let mut wave = W::new();
        for note in notes {
            let sound = self.play_note(*note);
            wave.add_consuming(sound, self.time_keeper.stamp_to_samples(note.on));
        }
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn play_test_chord(&self) -> W {
        let note_on = self.time_keeper.zero();
        let note_off = self.time_keeper.seconds_to_stamp(2.0);
        let mut out = self.play_freq(note_on, note_off, 300.0, 0.7);
        out.add_consuming(self.play_freq(note_on, note_off, 375.0, 0.7), 0);
        out.add_consuming(self.play_freq(note_on, note_off, 450.0, 0.7), 0);
        out.add_consuming(self.play_freq(note_on, note_off, 600.0, 0.7), 0);
        out
    }

    pub fn save_test_chord(&self) {
        let track = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        let path = Path::new(&path);
        io::easy_save(track, path);
    }
}
