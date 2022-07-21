use super::MidiInstrument;
use crate::{
    control::{Control, ControlError, Source, FunctionKeeper},
    ctrl_f::{
        self, Constant, CtrlFunction, Envelope, FunctionManager, FunctionMngrKeeper, FunctionOwner,
        IdMap, IdMapOrErr, Lfo,
    },
    effects::EffectPanel,
    time::{self, TimeKeeper, TimeManager, TimeStamp},
    tracks::midi,
    utils::{self, oscs::Oscillator},
    wave::Wave,
};
use std::{
    cell::RefCell, collections::HashMap, marker::PhantomData, path::Path, rc::Rc, result::Result,
};

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

impl<W: Wave> TimeKeeper for OscPanel<W> {
    fn set_time_manager(&mut self, _time_manager: Rc<RefCell<TimeManager>>) {}
}

impl<W: Wave> FunctionKeeper for OscPanel<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        for w in &mut self.weights {
            w.heal_sources(id_map)
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        for p in &mut self.pitch_offsets {
            p.heal_sources(id_map)
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        Ok(())
    }

    fn set_ids(&mut self) {
        for w in &mut self.weights {
            w.set_ids()
        }
        for p in &mut self.pitch_offsets {
            p.set_ids()
        }
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        for w in &self.weights {
            w.test_sources()
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        for p in &self.pitch_offsets {
            p.test_sources()
                .map_err(|err| err.push_location("OscPanel"))?;
        }
        Ok(())
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        for w in &self.weights {
            ids.append(&mut w.get_ids())
        }
        for p in &self.pitch_offsets {
            ids.append(&mut p.get_ids())
        }
        ids
    }
}

#[derive(Debug)]
struct LocalFManager {
    main_envelope: Rc<RefCell<Envelope>>,
    alt_envelope: Rc<RefCell<Envelope>>,
    current_velocity: Rc<RefCell<Constant>>,
    lfo1: Rc<RefCell<Lfo>>,
    lfo2: Rc<RefCell<Lfo>>,
    track_functions: Rc<RefCell<FunctionManager>>,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl LocalFManager {
    fn new() -> Self {
        Self {
            main_envelope: Rc::new(RefCell::new(Envelope::default())),
            alt_envelope: Rc::new(RefCell::new(Envelope::default())),
            current_velocity: Rc::new(RefCell::new(Constant::default())),
            lfo1: Rc::new(RefCell::new(Lfo::default())),
            lfo2: Rc::new(RefCell::new(Lfo::default())),
            track_functions: Rc::new(RefCell::new(FunctionManager::new())),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl LocalFManager {
    pub fn set_velocity(&mut self, velocity: f64) {
        self.current_velocity.borrow_mut().set(velocity)
    }
}

impl TimeKeeper for LocalFManager {
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

impl FunctionKeeper for LocalFManager {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.main_envelope
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.alt_envelope
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.current_velocity
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo1
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo2
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("LocalFManager"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.main_envelope
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.alt_envelope
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.current_velocity
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo1
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))?;
        self.lfo2
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("LocalFManager"))
    }

    fn set_ids(&mut self) {
        self.main_envelope.borrow_mut().set_ids();
        self.alt_envelope.borrow_mut().set_ids();
        self.current_velocity.borrow_mut().set_ids();
        self.lfo1.borrow_mut().set_ids();
        self.lfo2.borrow_mut().set_ids()
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.main_envelope.borrow().get_ids());
        ids.append(&mut self.alt_envelope.borrow().get_ids());
        ids.append(&mut self.current_velocity.borrow().get_ids());
        ids.append(&mut self.lfo1.borrow().get_ids());
        ids.append(&mut self.lfo2.borrow().get_ids());
        ids
    }
}

impl FunctionOwner for LocalFManager {
    unsafe fn new_ids(&mut self) {
        self.main_envelope.borrow_mut().new_id_f();
        self.alt_envelope.borrow_mut().new_id_f();
        self.current_velocity.borrow_mut().new_id_f();
        self.lfo1.borrow_mut().new_id_f();
        self.lfo2.borrow_mut().new_id_f();
        // new ids for the track_functions are set in the Track struct
    }

    fn get_id_map(&self) -> IdMapOrErr {
        let mut map = HashMap::<usize, Rc<RefCell<dyn CtrlFunction>>>::new();

        let main_envelope = ctrl_f::make_ctrl_function(Rc::clone(&self.main_envelope));
        let alt_envelope = ctrl_f::make_ctrl_function(Rc::clone(&self.alt_envelope));
        let lfo1 = ctrl_f::make_ctrl_function(Rc::clone(&self.lfo1));
        let lfo2 = ctrl_f::make_ctrl_function(Rc::clone(&self.lfo2));
        let current_velocity = ctrl_f::make_ctrl_function(Rc::clone(&self.current_velocity));

        ctrl_f::try_insert_id(main_envelope, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(alt_envelope, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(lfo1, &mut map).map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(lfo2, &mut map).map_err(|err| err.push_location("LocalFManager"))?;
        ctrl_f::try_insert_id(current_velocity, &mut map)
            .map_err(|err| err.push_location("LocalFManager"))?;
        // track_functions are inserted in the Track struct

        Ok(map)
    }
}

impl FunctionMngrKeeper for LocalFManager {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>) {
        self.track_functions = Rc::clone(&function_manager)
    }
}

#[derive(Debug)]
pub struct Synthesizer<W: Wave> {
    name: String,
    effects: EffectPanel<W>,
    oscillators: OscPanel<W>,
    fuctions: Rc<RefCell<LocalFManager>>,
    pitch_control: Control,
    modulation_control: Control,
    volume_control: Control,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl<W: Wave> Synthesizer<W> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: EffectPanel::EmptyLeaf,
            fuctions: Rc::new(RefCell::new(LocalFManager::new())),
            pitch_control: Control::from_val_in_unit(0.5).unwrap(),
            modulation_control: Control::from_val_in_unit(0.5).unwrap(),
            volume_control: Control::from_val_in_unit(1.0).unwrap(),
            oscillators: OscPanel::default(),
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        }
    }
}

impl<W: Wave> time::TimeKeeper for Synthesizer<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.effects.set_time_manager(Rc::clone(&time_manager));
        self.fuctions
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
        self.fuctions.borrow_mut().set_velocity(velocity);
        let sus_samples = self
            .time_manager
            .borrow()
            .duration_to_samples(note_on, note_off);

        let envelope = self
            .fuctions
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
        self.effects.apply_to(&mut wave, note_on);
        wave
    }
}

impl<W: Wave> FunctionKeeper for Synthesizer<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.effects
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.oscillators
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.fuctions
            .borrow_mut()
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.pitch_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.modulation_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.volume_control
            .heal_sources(id_map)
            .map_err(|err| err.push_location("Synthesizer"))
    }

    fn set_ids(&mut self) {
        self.effects.set_ids();
        self.oscillators.set_ids();
        self.fuctions.borrow_mut().set_ids();
        self.pitch_control.set_ids();
        self.modulation_control.set_ids();
        self.volume_control.set_ids();
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.effects
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.oscillators
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.fuctions
            .borrow()
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.pitch_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.modulation_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))?;
        self.volume_control
            .test_sources()
            .map_err(|err| err.push_location("Synthesizer"))
    }

    fn get_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        ids.append(&mut self.effects.get_ids());
        ids.append(&mut self.oscillators.get_ids());
        ids.append(&mut self.fuctions.borrow().get_ids());
        ids.append(&mut self.pitch_control.get_ids());
        ids.append(&mut self.modulation_control.get_ids());
        ids.append(&mut self.volume_control.get_ids());
        ids
    }
}

impl<W: Wave> FunctionOwner for Synthesizer<W> {
    unsafe fn new_ids(&mut self) {
        self.fuctions.borrow_mut().new_ids()
    }

    fn get_id_map(&self) -> IdMapOrErr {
        self.fuctions.borrow().get_id_map()
    }
}

impl<W: Wave> FunctionMngrKeeper for Synthesizer<W> {
    fn set_fuction_manager(&mut self, function_manager: Rc<RefCell<FunctionManager>>) {
        self.fuctions
            .borrow_mut()
            .set_fuction_manager(Rc::clone(&function_manager))
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
    pub fn get_main_envelope(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().main_envelope,
        )))
    }

    pub fn get_alt_envelope(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().alt_envelope,
        )))
    }

    pub fn get_current_velocity(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().current_velocity,
        )))
    }

    pub fn get_lfo1(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().lfo1,
        )))
    }

    pub fn get_lfo2(&self) -> Source {
        Source::from_function(ctrl_f::make_ctrl_function(Rc::clone(
            &self.fuctions.borrow().lfo2,
        )))
    }

    pub fn get_automation_channel(&self, channel: u8) -> Option<Source> {
        self.fuctions
            .borrow()
            .track_functions
            .borrow()
            .get_channel(channel)
            .map(Source::from_function)
    }
}

impl<W: Wave> Synthesizer<W> {
    pub fn add_osc(&mut self, oscillator: Oscillator) {
        self.oscillators.add_osc(oscillator)
    }

    pub fn set_main_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .main_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_alt_envelope(&mut self, envelope: Envelope) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .alt_envelope
            .borrow_mut()
            .set(envelope)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo1(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .lfo1
            .borrow_mut()
            .set(lfo)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_lfo2(&mut self, lfo: Lfo) -> Result<(), ControlError> {
        self.fuctions
            .borrow_mut()
            .lfo2
            .borrow_mut()
            .set(lfo)
            .map_err(|err| err.push_location("Synthesizer"))?;
        Ok(())
    }

    pub fn set_vol_source(&mut self, source: Source) {
        self.volume_control = Control::new(1.0, VOL_CTRL_RANGE, source).unwrap()
    }

    pub fn set_pitch_source(&mut self, source: Source) {
        self.volume_control = Control::new(0.0, PITCH_WHEEL_RANGE, source).unwrap()
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
