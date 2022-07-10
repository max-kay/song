use super::MidiInstrument;
use crate::auto;
use crate::auto::CtrlVal;
use crate::effects;
use crate::io;
use crate::time;
use crate::time::TimeKeeper;
use crate::time::TimeManager;
use crate::tracks::midi;
use crate::utils::envelope;
use crate::utils::oscs;
use crate::wave;
use crate::wave::Wave;
use std::path::Path;
use std::rc::Rc;

pub struct SynthAutomation {
    main_envelope: envelope::Envelope,
    alt_envelope: envelope::Envelope,
    current_velocity: auto::Constant,
    lfo1: auto::Lfo,
    lfo2: auto::Lfo,
    track_automation: Rc<auto::AutomationManager>,
    time_manager: Rc<time::TimeManager>,
}

impl SynthAutomation {
    fn empty(time_manager: Rc<time::TimeManager>) -> Self {
        Self {
            main_envelope: envelope::Envelope::default(),
            alt_envelope: envelope::Envelope::default(),
            current_velocity: auto::Constant::default(),
            lfo1: auto::Lfo::default(),
            lfo2: auto::Lfo::default(),
            track_automation: Rc::new(auto::AutomationManager::new()),
            time_manager,
        }
    }

    pub fn get_main_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.main_envelope.get_envelope(sus_samples)
    }

    pub fn set_track_automation(&mut self, automation: &Rc<auto::AutomationManager>) {
        self.track_automation = Rc::clone(automation)
    }
}

impl TimeKeeper for SynthAutomation {
    fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.time_manager = Rc::clone(time_manager);
        self.lfo1.set_time_manager(time_manager);
        self.lfo2.set_time_manager(time_manager)
    }
}

pub struct Synthesizer<'a, W: Wave> {
    name: String,
    effects: effects::EffectNode<W>,
    effect_ctrl: effects::CtrlPanel<'a>,
    oscillators: Vec<Box<dyn oscs::Oscillator>>,
    local_automation: SynthAutomation,
    pitch_control: auto::Control,
    modulation_control: auto::Control,
    time_manager: Rc<time::TimeManager>,
    pitch_wheel_range: f64, // in cents
}

impl<'a, W: Wave> Synthesizer<'a, W> {
    pub fn new(name: String, oscillators: Vec<Box<dyn oscs::Oscillator>>) -> Self {
        Self {
            name,
            effects: effects::EffectNode::Bypass,
            effect_ctrl: effects::CtrlPanel::Bypass,
            local_automation: SynthAutomation::empty(Rc::new(time::TimeManager::default())),
            pitch_control: auto::Control::from_values(auto::CtrlVal::from_num(0.5_f32), 1.0),
            modulation_control: auto::Control::from_values(auto::CtrlVal::from_num(0_f32), 1.0),
            oscillators,
            pitch_wheel_range: 2.0,
            time_manager: Rc::new(time::TimeManager::default()),
        }
    }

    pub fn set_time_manager(&mut self, time_manager: &Rc<time::TimeManager>) {
        self.time_manager = Rc::clone(time_manager);
        self.local_automation.set_time_manager(time_manager)
    }
}

impl<'a, W: Wave> Synthesizer<'a, W> {
    fn get_envelope(&self, sus_samples: usize) -> Vec<f64> {
        self.local_automation.get_main_envelope(sus_samples)
    }
    fn play_freq(
        &self,
        note_on: time::TimeStamp,
        note_off: time::TimeStamp,
        freq: f64,
        velocity: auto::CtrlVal,
    ) -> W {
        let velocity = auto::Constant(velocity);
        let sus_samples = self.time_manager.duration_to_samples(note_on, note_off);
        let envelope = self.get_envelope(sus_samples);
        let freq: Vec<f64> = self
            .pitch_control
            .get_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf((x * 2.0 - 1.0) * self.pitch_wheel_range / 1200.0))
            .collect();
        let modulation = self.modulation_control.get_vec(note_on, envelope.len());
        let mut wave = W::zeros(envelope.len());
        for osc in &self.oscillators {
            wave.add_consuming(W::from_vec(osc.play(&freq, &modulation, envelope.len())), 0);
        }
        wave.scale_by_vec(envelope);
        self.effects.apply(&mut wave, &self.effect_ctrl, note_on);
        wave
    }
}

impl<W: wave::Wave> time::TimeKeeper for Synthesizer<'_, W> {
    fn set_time_manager(&mut self, time_manager: &Rc<TimeManager>) {
        self.effects.set_time_manager(time_manager);
        self.effect_ctrl.set_time_manager(time_manager);
        self.local_automation.set_time_manager(time_manager);
        self.pitch_control.set_time_manager(time_manager);
        self.modulation_control.set_time_manager(time_manager)
    }
}

impl<'a, W: Wave> MidiInstrument<W> for Synthesizer<'a, W> {
    fn play_note(&self, note: midi::Note) -> W {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &[midi::Note]) -> W {
        let mut wave = W::new();
        for note in notes {
            let sound = self.play_note(*note);
            wave.add_consuming(sound, self.time_manager.stamp_to_samples(note.on));
        }
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn set_track_automation(&mut self, automation: &Rc<auto::AutomationManager>) {
        self.local_automation.set_track_automation(automation)
    }
}

impl<'a, W: Wave> Synthesizer<'a, W> {
    pub fn play_test_chord(&self) -> W {
        let note_on = self.time_manager.zero();
        let note_off = self.time_manager.seconds_to_stamp(2.0);
        let mut out = self.play_freq(note_on, note_off, 300.0, CtrlVal::from_num(0.7_f64));
        out.add_consuming(
            self.play_freq(note_on, note_off, 375.0, CtrlVal::from_num(0.7_f64)),
            0,
        );
        out.add_consuming(
            self.play_freq(note_on, note_off, 450.0, CtrlVal::from_num(0.7_f64)),
            0,
        );
        out.add_consuming(
            self.play_freq(note_on, note_off, 600.0, CtrlVal::from_num(0.7_f64)),
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
