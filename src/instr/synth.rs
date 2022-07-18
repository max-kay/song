use super::MidiInstrument;
use crate::{
    auto::{
        self, AutomationKeeper, AutomationManager, Constant, Control, CtrlFunction, Envelope, Lfo, ControlError,
    },
    effects,
    time::{self, TimeKeeper, TimeManager, TimeStamp},
    tracks::midi,
    utils::{self, oscs::Oscillator},
    wave::Wave,
};
use std::{cell::RefCell, marker::PhantomData, path::Path, rc::Rc};

const PITCH_WHEEL_RANGE: (f64, f64) = (-4800.0, 4800.0);
const VOL_CTRL_RANGE: (f64, f64) = (0.0, 5.0);

#[derive(Debug)]
pub struct OscPanel<W: Wave> {
    phantom: PhantomData<W>,
    oscillators: Vec<Oscillator>,
    weights: Vec<Control>,
    pitch_offsets: Vec<Control>,
}

impl<W: Wave> Default for OscPanel<W> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::<W>,
            oscillators: vec![Oscillator::default()],
            weights: vec![Control::from_val_in_unit(1.0).unwrap()],
            pitch_offsets: vec![Control::from_val_in_range(0.0, (-4800.0, 4800.0)).unwrap()],
        }
    }
}

impl<W: Wave> OscPanel<W> {
    pub fn play(
        &self,
        freq: Vec<f64>,
        modulation: Vec<f64>,
        start: TimeStamp,
        samples: usize,
    ) -> W {
        let mut wave = vec![0.0; samples];

        for ((osc, weigth), offset) in self
            .oscillators
            .iter()
            .zip(&self.weights)
            .zip(&self.pitch_offsets)
        {
            let freq = offset
                .get_vec(start, samples)
                .into_iter()
                .zip(&freq)
                .map(|(x, y)| y * 2_f64.powf(x / 1200.0))
                .collect();

            let new_wave = osc
                .play(&freq, &modulation, samples)
                .into_iter()
                .zip(weigth.get_vec(start, samples))
                .map(|(x, y)| x * y)
                .collect();

            utils::add_elementwise(&mut wave, new_wave)
        }
        W::from_vec(wave)
    }

    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.push(oscillator);
        self.pitch_offsets
            .push(Control::from_val_in_range(0.0, PITCH_WHEEL_RANGE).unwrap());
        self.weights.push(Control::from_val_in_unit(1.0).unwrap());
    }
}

#[derive(Debug)]
pub struct SynthAutomation {
    main_envelope: Rc<RefCell<Envelope>>,
    alt_envelope: Rc<RefCell<Envelope>>,
    current_velocity: Rc<RefCell<Constant>>,
    lfo1: Rc<RefCell<Lfo>>,
    lfo2: Rc<RefCell<Lfo>>,
    track_automation: Rc<RefCell<AutomationManager>>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl SynthAutomation {
    fn new() -> Self {
        Self {
            main_envelope: Rc::new(RefCell::new(Envelope::default())),
            alt_envelope: Rc::new(RefCell::new(Envelope::default())),
            current_velocity: Rc::new(RefCell::new(Constant::default())),
            lfo1: Rc::new(RefCell::new(Lfo::default())),
            lfo2: Rc::new(RefCell::new(Lfo::default())),
            track_automation: Rc::new(RefCell::new(AutomationManager::new())),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl SynthAutomation {
    pub fn set_velocity(&mut self, velocity: f64) {
        (*self.current_velocity).borrow_mut().set(velocity)
    }
}

impl TimeKeeper for SynthAutomation {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager);
        self.lfo1
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.lfo2
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.main_envelope
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.alt_envelope
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager))
    }
}

impl AutomationKeeper for SynthAutomation {
    fn set_automation_manager(&mut self, automation_manager: Rc<RefCell<AutomationManager>>) {
        self.track_automation = Rc::clone(&automation_manager)
    }
}

#[derive(Debug)]
pub struct Synthesizer<W: Wave> {
    name: String,
    effects: effects::EffectNode<W>,
    oscillators: OscPanel<W>,
    local_automation: Rc<RefCell<SynthAutomation>>,
    pitch_control: auto::Control,
    modulation_control: auto::Control,
    volume_control: auto::Control,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: effects::EffectNode::Bypass,
            local_automation: Rc::new(RefCell::new(SynthAutomation::new())),
            pitch_control: auto::Control::from_val_in_unit(0.5).unwrap(),
            modulation_control: auto::Control::from_val_in_unit(0.5).unwrap(),
            volume_control: auto::Control::from_val_in_unit(1.0).unwrap(),
            oscillators: OscPanel::default(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl<W: Wave> time::TimeKeeper for Synthesizer<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.effects.set_time_manager(Rc::clone(&time_manager));
        (*self.local_automation)
            .borrow_mut()
            .set_time_manager(Rc::clone(&time_manager));
        self.pitch_control
            .set_time_manager(Rc::clone(&time_manager));
        self.modulation_control
            .set_time_manager(Rc::clone(&time_manager))
    }
}

impl<W: Wave> Synthesizer<W> {
    fn play_freq(&self, note_on: TimeStamp, note_off: TimeStamp, freq: f64, velocity: f64) -> W {
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
            .get_envelope(sus_samples, note_on);

        let freq: Vec<f64> = self
            .pitch_control
            .get_vec(note_on, envelope.len())
            .into_iter()
            .map(|x| freq * 2_f64.powf(x / 1200.0))
            .collect();

        let modulation = self.modulation_control.get_vec(note_on, envelope.len());
        let mut wave = W::zeros(envelope.len());
        wave.add_consuming(
            self.oscillators
                .play(freq, modulation, note_on, envelope.len()),
            0,
        );
        wave.scale_by_vec(self.volume_control.get_vec(note_on, envelope.len()));
        wave.scale_by_vec(envelope);
        self.effects.apply(&mut wave, note_on);
        wave
    }
}

impl<W: Wave> AutomationKeeper for Synthesizer<W> {
    fn set_automation_manager(&mut self, automation_manager: Rc<RefCell<AutomationManager>>) {
        self.local_automation
            .borrow_mut()
            .set_automation_manager(Rc::clone(&automation_manager))
    }
}

impl<W: Wave> MidiInstrument<W> for Synthesizer<W> {
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

impl<W: Wave> Synthesizer<W> {
    pub fn get_main_envelope(&self) -> Rc<RefCell<dyn CtrlFunction>> {
        auto::make_ctrl_function(Rc::clone(&self.local_automation.borrow().main_envelope))
    }

    pub fn get_alt_envelope(&self) -> Rc<RefCell<dyn CtrlFunction>> {
        auto::make_ctrl_function(Rc::clone(&self.local_automation.borrow().alt_envelope))
    }

    pub fn get_current_velocity(&self) -> Rc<RefCell<dyn CtrlFunction>> {
        auto::make_ctrl_function(Rc::clone(&self.local_automation.borrow().current_velocity))
    }

    pub fn get_lfo1(&self) -> Rc<RefCell<dyn CtrlFunction>> {
        auto::make_ctrl_function(Rc::clone(&self.local_automation.borrow().lfo1))
    }

    pub fn get_lfo2(&self) -> Rc<RefCell<dyn CtrlFunction>> {
        auto::make_ctrl_function(Rc::clone(&self.local_automation.borrow().lfo2))
    }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Rc<RefCell<dyn CtrlFunction>>> {
        self.local_automation
            .borrow()
            .track_automation
            .borrow()
            .get_channel(channel)
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.add_osc(oscillator)
    }

    pub fn set_main_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.local_automation
            .borrow_mut()
            .main_envelope
            .borrow_mut()
            .set(envelope)?;
            Ok(())
    }

    pub fn set_alt_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.local_automation
            .borrow_mut()
            .alt_envelope
            .borrow_mut()
            .set(envelope)?;
        Ok(())
    }

    pub fn set_lfo1(&mut self, lfo: Lfo) -> Result<(), ControlError>{
        self.local_automation
            .borrow_mut()
            .lfo1
            .borrow_mut()
            .set(lfo)?;
        Ok(())
    }

    pub fn set_lfo2(&mut self, lfo: Lfo) -> Result<(), ControlError>{
        self.local_automation
            .borrow_mut()
            .lfo2
            .borrow_mut()
            .set(lfo)?;
        Ok(())
    }

    pub fn set_vol_control(&mut self, ctrl_func: Rc<RefCell<dyn CtrlFunction>>) {
        self.volume_control = Control::new(1.0, VOL_CTRL_RANGE, ctrl_func).unwrap()
    }

    pub fn set_pitch_control(&mut self, ctrl_func: Rc<RefCell<dyn CtrlFunction>>) {
        self.volume_control = Control::new(0.0, PITCH_WHEEL_RANGE, ctrl_func).unwrap()
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn play_test_chord(&self) -> W {
        let note_on = self.time_manager.borrow().zero();
        let note_off = self.time_manager.borrow().seconds_to_stamp(6.0);
        let mut wave = self.play_freq(note_on, note_off, 300.0, 0.7);
        wave.add_consuming(self.play_freq(note_on, note_off, 375.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 450.0, 0.7), 0);
        wave.add_consuming(self.play_freq(note_on, note_off, 600.0, 0.7), 0);
        wave
    }

    pub fn save_test_chord(&self) {
        let wave = self.play_test_chord();
        let path = format!("out/synthtest/{}_chord.wav", self.name);
        let path = Path::new(&path);
        wave.save(path);
    }
}
