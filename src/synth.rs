use crate::auto::{AutomationManager, Lfo, PointDefined, TimeFunction, ValAndCh, ValOrVec};
use crate::envelope::Envelope;
use crate::midi;
use crate::song::{self, Instrument};
use crate::time::{Duration, TimeStamp};
use crate::utils::add_same_len;
use crate::{io, oscs};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

const LFO1: u8 = 0;
const LFO2: u8 = 1;
const PITCHBEND: u8 = 2;
const MOD_WHEEL: u8 = 3;
const HARD_CODED_CHANNELS: [u8; 4] = [LFO1, LFO2, PITCHBEND, MOD_WHEEL];

pub struct LocalAutomation {
    channels: HashMap<u8, Box<dyn TimeFunction>>,
    main_envelope: Envelope,
    alt_envelope: Envelope,
}

impl LocalAutomation {
    fn empty() -> Self {
        let mut channels = HashMap::<u8, Box<dyn TimeFunction>>::new();
        channels.insert(LFO1, Box::new(Lfo::default_lfo())); // lfo1
        channels.insert(LFO2, Box::new(Lfo::default_lfo())); // lfo2
        channels.insert(PITCHBEND, Box::new(PointDefined::one_point(0.5))); // pitchbend
        channels.insert(MOD_WHEEL, Box::new(PointDefined::one_point(0.5))); // modulation_wheel
        Self {
            channels,
            main_envelope: Envelope::default(),
            alt_envelope: Envelope::default(),
        }
    }

    pub fn get_main_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.main_envelope.get_envelope(sus_samples)
    }

    pub fn get_pitch_vec(&self, onset: TimeStamp, samples: usize) -> Vec<f64> {
        self.channels
            .get(&PITCHBEND)
            .expect("Error while getting pitchbendvec")
            .get_vec(onset, samples)
    }

    pub fn get_channel(&self, channel: u8) -> Option<&dyn TimeFunction> {
        match self.channels.get(&channel) {
            Some(timefunction) => Some(&**timefunction),
            None => None,
        }
    }

    pub fn set_channel(&mut self, timefunction: Box<dyn TimeFunction>, channel: u8) {
        assert!(!HARD_CODED_CHANNELS.contains(&channel), "channel 0 to 3 are used as defaults for lfo1, lfo2, pitchbend, modulation_wheel in that order");
        self.channels.insert(channel, timefunction);
    }
}

pub struct Connections {
    connections: Vec<ValAndCh>,
}

pub struct Synthesizer {
    name: String,
    oscillators: Vec<Box<dyn oscs::Oscillator>>,
    local_automation: LocalAutomation,
    global_automation: Rc<AutomationManager>,
    connections: Connections,
    pitch_wheel_range: f64, // in cents
}

impl Synthesizer {
    pub fn new(
        name: String,
        oscillators: Vec<Box<dyn oscs::Oscillator>>,
        local_automation: LocalAutomation,
        global_automation: Rc<AutomationManager>,
        connections: Connections,
        pitch_wheel_range: f64,
    ) -> Self {
        Self {
            name,
            oscillators,
            local_automation,
            global_automation,
            connections,
            pitch_wheel_range,
        }
    }

    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.local_automation.get_main_envelope(sus_samples)
    }
}

impl song::Instrument for Synthesizer {
    fn play_midi_note(&self, note: midi::Note) -> Vec<f64> {
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
    ) -> Vec<f64> {
        let sus_samples = note_held.to_samples();
        let envelope = self.get_envelope(sus_samples);
        let freq = self
            .local_automation
            .get_pitch_vec(onset, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf((x * 2.0 - 1.0) * self.pitch_wheel_range / 1200.0))
            .collect();
        let freq = ValOrVec::Vec(freq);
        let modulation = ValOrVec::Val(0.5);
        let mut out = vec![0.0; envelope.len()];
        for osc in &self.oscillators {
            add_same_len(&mut out, osc.wave(&freq, &modulation, envelope.len()));
        }
        add_same_len(&mut out, envelope);
        out
    }
}

impl Synthesizer {
    pub fn play_test_chord(&self) -> Vec<f64> {
        let time = TimeStamp::zero();
        let duration = Duration::one_sec();
        let mut out = self.play_freq(time, duration, 300.0, midi::Velocity::new(80).unwrap());
        add_same_len(
            &mut out,
            self.play_freq(time, duration, 375.0, midi::Velocity::new(80).unwrap()),
        );
        add_same_len(
            &mut out,
            self.play_freq(time, duration, 450.0, midi::Velocity::new(80).unwrap()),
        );
        add_same_len(
            &mut out,
            self.play_freq(time, duration, 600.0, midi::Velocity::new(80).unwrap()),
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
