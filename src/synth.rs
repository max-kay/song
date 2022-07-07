use crate::auto::{Lfo, PointDefined, TimeFunction, Control, ValOrVec};
use crate::envelope::Envelope;
use crate::midi;
use crate::song::{self, Instrument};
use crate::time::{Duration, TimeKeeper, TimeStamp};
use crate::wave::Wave;
use crate::{io, oscs};
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

pub struct SynthAutomation {
    lfo1: Lfo,
    lfo2: Lfo,
    pitchbend: PointDefined,
    modulation_wheel: PointDefined,
    main_envelope: Envelope,
    alt_envelope: Envelope,
    time_keeper: Rc<TimeKeeper>,
}

impl SynthAutomation {
    fn empty(time_keeper: Rc<TimeKeeper>) -> Self {
        let lfo1 = Lfo::default_lfo();
        let lfo2 = Lfo::default_lfo();
        let pitchbend = PointDefined::one_point(0.5, Rc::clone(&time_keeper));
        let modulation_wheel = PointDefined::one_point(0.5, Rc::clone(&time_keeper));
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

    pub fn get_pitch_vec(&self, onset: TimeStamp, samples: usize) -> Vec<f64> {
        self.pitchbend.get_vec(onset, samples)
    }
}

pub struct Connections {
    connections: Vec<Control>,
}

pub struct Synthesizer<W: Wave> {
    phantom: PhantomData<W>,
    name: String,
    oscillators: Vec<Box<dyn oscs::Oscillator<W>>>,
    local_automation: SynthAutomation,
    connections: Connections,
    pitch_wheel_range: f64, // in cents
    time_keeper: Rc<TimeKeeper>,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(
        name: String,
        oscillators: Vec<Box<dyn oscs::Oscillator<W>>>,
        local_automation: SynthAutomation,
        connections: Connections,
        time_keeper: Rc<TimeKeeper>,
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

impl<W: Wave> song::Instrument<W> for Synthesizer<W> {
    fn play_midi_note(&self, note: midi::Note) -> W {
        self.play_freq(
            note.onset,
            note.note_held,
            note.pitch.get_freq(),
            note.velocity,
        )
    }

    fn play_freq(
        &self,
        onset: TimeStamp,
        note_held: Duration,
        freq: f64,
        velocity: midi::Velocity,
    ) -> W {
        let sus_samples = self.time_keeper.duration_to_samples(note_held, onset);
        let envelope = self.get_envelope(sus_samples);
        let freq = self
            .local_automation
            .get_pitch_vec(onset, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf((x * 2.0 - 1.0) * self.pitch_wheel_range / 1200.0))
            .collect();
        let freq = ValOrVec::Vec(freq);
        let modulation = ValOrVec::Val(0.5);
        let mut wave = W::zeros(envelope.len());
        for osc in &self.oscillators {
            wave.add_consuming(osc.wave(&freq, &modulation, envelope.len()), 0);
        }
        wave.scale_by_vec(envelope);
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn play_test_chord(&self) -> W {
        let time = TimeStamp::zero();
        let duration = self.time_keeper.duration_from_seconds(2.5, time);
        let mut out = self.play_freq(time, duration, 300.0, midi::Velocity::new(80).unwrap());
        out.add_consuming(
            self.play_freq(time, duration, 375.0, midi::Velocity::new(80).unwrap()),
            0,
        );
        out.add_consuming(
            self.play_freq(time, duration, 450.0, midi::Velocity::new(80).unwrap()),
            0,
        );
        out.add_consuming(
            self.play_freq(time, duration, 600.0, midi::Velocity::new(80).unwrap()),
            0,
        );
        out
    }

    pub fn save_test_chord(&self) {
        let track = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        let path = Path::new(&path);
        io::easy_save(track, path);
    }
}
