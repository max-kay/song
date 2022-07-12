use super::MidiInstrument;
use crate::{
    auto::{self, AutomationManager, CtrlFunction},
    effects, io,
    time::{self, TimeKeeper},
    tracks::midi,
    utils::oscs,
    wave::{self, Wave},
};
use std::{cell::RefCell, path::Path, rc::Rc};

#[derive(Debug)]
pub struct SynthAutomation {
    main_envelope: Rc<RefCell<dyn auto::Envelope>>,
    alt_envelope: Rc<RefCell<dyn auto::Envelope>>,
    current_velocity: Rc<RefCell<auto::Constant>>,
    lfo1: Rc<RefCell<auto::Lfo>>,
    lfo2: Rc<RefCell<auto::Lfo>>,
    track_automation: Rc<RefCell<auto::AutomationManager>>,
    time_manager: Rc<RefCell<time::TimeManager>>,
}

impl SynthAutomation {
    fn new() -> Self {
        Self {
            main_envelope: Rc::new(RefCell::new(auto::Adsr::default())),
            alt_envelope: Rc::new(RefCell::new(auto::Adsr::default())),
            current_velocity: Rc::new(RefCell::new(auto::Constant::default())),
            lfo1: Rc::new(RefCell::new(auto::Lfo::default())),
            lfo2: Rc::new(RefCell::new(auto::Lfo::default())),
            track_automation: Rc::new(RefCell::new(auto::AutomationManager::new())),
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl SynthAutomation {
    pub fn set_velocity(&mut self, velocity: f64) {
        (*self.current_velocity).borrow_mut().set(velocity)
    }
}

impl TimeKeeper for SynthAutomation {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager);
        (*self.lfo1)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        (*self.lfo2)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        (*self.main_envelope)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        (*self.alt_envelope)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager))
    }
}

impl auto::AutomationKeeper for SynthAutomation {
    fn set_automation_manager(&mut self, automation_manager: Rc<RefCell<AutomationManager>>) {
        self.track_automation = Rc::clone(&automation_manager)
    }
}

#[derive(Debug)]
pub struct Synthesizer<'a, W: Wave> {
    name: String,
    effects: effects::EffectNode<W>,
    effect_ctrl: effects::CtrlPanel<'a>,
    oscillators: Vec<Box<dyn oscs::Oscillator>>,
    local_automation: Rc<RefCell<SynthAutomation>>,
    pitch_control: auto::Control,
    modulation_control: auto::Control,
    volume_control: auto::Control,
    time_manager: Rc<RefCell<time::TimeManager>>,
    pitch_wheel_range: f64, // in cents
}

impl<W: Wave> Synthesizer<'_, W> {
    pub fn new(name: String, oscillators: Vec<Box<dyn oscs::Oscillator>>) -> Self {
        Self {
            name,
            effects: effects::EffectNode::Bypass,
            effect_ctrl: effects::CtrlPanel::Bypass,
            local_automation: Rc::new(RefCell::new(SynthAutomation::new())),
            pitch_control: auto::Control::val_in_unit(0.5),
            modulation_control: auto::Control::val_in_unit(0.5),
            volume_control: auto::Control::val_in_unit(1.0),
            oscillators,
            pitch_wheel_range: 2.0,
            time_manager: Rc::new(RefCell::new(time::TimeManager::default())),
        }
    }
}

impl<W: wave::Wave> time::TimeKeeper for Synthesizer<'_, W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<time::TimeManager>>) {
        self.effects.set_time_manager(Rc::clone(&time_manager));
        self.effect_ctrl.set_time_manager(Rc::clone(&time_manager));
        (*self.local_automation)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.pitch_control
            .set_time_manager(Rc::clone(&time_manager));
        self.modulation_control
            .set_time_manager(Rc::clone(&time_manager))
    }
}

impl<W: Wave> Synthesizer<'_, W> {
    fn play_freq(
        &self,
        note_on: time::TimeStamp,
        note_off: time::TimeStamp,
        freq: f64,
        velocity: f64,
    ) -> W {
        (*self.local_automation).borrow_mut().set_velocity(velocity);
        let sus_samples = self
            .time_manager
            .borrow()
            .duration_to_samples(note_on, note_off);

        let envelope = self
            .local_automation
            .borrow()
            .main_envelope
            .borrow()
            .get_envelope(sus_samples);

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
        wave.scale_by_vec(self.volume_control.get_vec(note_on, envelope.len()));
        wave.scale_by_vec(envelope);
        self.effects.apply(&mut wave, &self.effect_ctrl, note_on);
        wave
    }
}

impl<W: Wave> auto::AutomationKeeper for Synthesizer<'_, W> {
    fn set_automation_manager(&mut self, automation_manager: Rc<RefCell<auto::AutomationManager>>) {
        (*self.local_automation)
            .borrow_mut()
            .set_automation_manager(Rc::clone(&automation_manager))
    }
}

impl<W: Wave> MidiInstrument<W> for Synthesizer<'_, W> {
    fn play_note(&self, note: midi::Note) -> W {
        self.play_freq(note.on, note.off, note.pitch.get_freq(), note.velocity)
    }
    fn play_notes(&self, notes: &[midi::Note]) -> W {
        let mut wave = W::new();
        for note in notes {
            let sound = self.play_note(*note);
            wave.add_consuming(sound, self.time_manager.borrow().stamp_to_samples(note.on));
        }
        wave
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<W: wave::Wave> Synthesizer<'_, W> {
    // pub fn get_main_envelope(&self) -> Rc<RefCell<dyn CtrlFunction>> {
    //     todo!()
    // }

    // pub fn get_alt_envelope(&self) -> Rc<RefCell<dyn CtrlFunction>> {
    //     todo!()
    // }

    // pub fn get_current_velocity(&self) -> Rc<RefCell<dyn CtrlFunction>> {
    //     Rc::clone(&(self.local_automation.borrow().current_velocity))
    // }

    // pub fn get_lfo1(&self) -> Rc<RefCell<dyn CtrlFunction>> {
    //     Rc::clone(&(self.local_automation.borrow().lfo1 as Rc<RefCell<dyn CtrlFunction>>))
    // }

    // pub fn get_lfo2(&self) -> Rc<RefCell<dyn CtrlFunction>> {
    //     Rc::clone(&(self.local_automation.borrow().lfo2 as Rc<RefCell<dyn CtrlFunction>>))
    // }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Rc<RefCell<dyn CtrlFunction>>> {
        self.local_automation
            .borrow()
            .track_automation
            .borrow()
            .get_channel(channel)
    }
}

impl<'a, W: Wave> Synthesizer<'a, W> {
    pub fn play_test_chord(&self) -> W {
        let note_on = self.time_manager.borrow().zero();
        let note_off = self.time_manager.borrow().seconds_to_stamp(2.0);
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
