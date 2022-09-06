use midly::{num::u7, Format, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use serde::{Deserialize, Serialize};

use crate::{
    ctrl_f::{
        point_defined::{AutomationPoint, Interpolation},
        Generator, GeneratorManager, PointDefined, SaveId,
    },
    globals::{GENRATOR_MANAGER, TIME_MANAGER},
    time::{TimeManager, TimeSignature, TimeStamp},
    tracks::{
        midi::{self, MidiTrack},
        Track,
    },
    wave::Wave,
    Song,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fs,
    path::Path,
};

pub fn save_wave(wave: Wave, path: &Path) {
    wave.save(path)
}

pub fn parse_midi_file(path: &Path) -> Result<SongData, Box<dyn Error>> {
    let bytes = fs::read(&path)?;
    let smf = Smf::parse(&bytes)?;
    let mut time_decoder = TimeDecoder::new(smf.header.timing);
    if smf.header.format == Format::Sequential {
        todo!()
    }
    let mut almost_track = Vec::new();
    for (i, track) in smf.tracks.iter().enumerate() {
        almost_track.push(parse_midi_track(
            track,
            &mut time_decoder,
            i.try_into().unwrap(),
        ))
    }
    time_decoder
        .validate()
        .expect("time decoding error (validation)");
    let mut decoded = SongData::new();
    for track in almost_track {
        decoded
            .add_track_data(track.into_track(&time_decoder))
            .expect("time decoding error in track");
    }
    Ok(decoded)
}

fn parse_midi_track(
    track: &Vec<TrackEvent>,
    time_decoder: &mut TimeDecoder,
    track_index: u16,
) -> AlmostTrack {
    let mut data = AlmostTrack::new(track_index);
    let mut current_ticks = 0;
    for event in track {
        current_ticks += event.delta.as_int();
        match event.kind {
            TrackEventKind::Meta(msg) => {
                decode_meta_msg(msg, &mut data, time_decoder, current_ticks)
            }
            TrackEventKind::Midi {
                channel: _,
                message,
            } => match message {
                MidiMessage::NoteOn { key, vel } => data.push_note_on(current_ticks, key, vel),
                MidiMessage::NoteOff { key, vel: _ } => data.push_note_off(current_ticks, key),
                MidiMessage::Controller { controller, value } => {
                    data.push_cc(current_ticks, controller, value)
                }
                MidiMessage::PitchBend { bend } => data.push_pitch_bend(current_ticks, bend),
                MidiMessage::Aftertouch { key, vel } => {
                    data.push_after_touch(current_ticks, key, vel)
                }
                MidiMessage::ChannelAftertouch { vel } => {
                    data.push_ch_after_touch(current_ticks, vel)
                }
                MidiMessage::ProgramChange { program: _ } => (),
            },
            TrackEventKind::SysEx(_) => (),
            TrackEventKind::Escape(_) => (),
        }
    }
    data
}

fn decode_meta_msg(
    msg: midly::MetaMessage,
    data: &mut AlmostTrack,
    time_decoder: &mut TimeDecoder,
    current_ticks: u32,
) {
    use midly::MetaMessage::*;
    match msg {
        TrackNumber(opt) => {
            if let Some(val) = opt {
                data.change_number(val);
            }
        }
        TrackName(name) => data
            .change_name(&String::from_utf8(name.to_vec()).expect("recieved invalid track name")),
        InstrumentName(name) => data.change_inst_name(
            &String::from_utf8(name.to_vec()).expect("recieved invalid instrument name"),
        ),
        Tempo(tempo) => time_decoder.push_tempo(current_ticks, tempo.as_int()),
        TimeSignature(num, den, _, _) => time_decoder.push_signature(current_ticks, num, den),
        EndOfTrack => (),
        KeySignature(_, _) => (),
        MidiChannel(_) => (),
        SmpteOffset(_) => todo!(),
        Text(txt) => println!(
            "ignored text meta message:{:?}",
            String::from_utf8(txt.to_vec())
        ),
        Copyright(txt) => println!(
            "ignored text meta message:{:?}",
            String::from_utf8(txt.to_vec())
        ),
        _ => println!("ignored {:?}", msg),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongData {
    name: String,
    tracks: HashMap<u8, Track>,
    time_manager: TimeManager,
    generator_manager: GeneratorManager,
}

impl SongData {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            tracks: HashMap::new(),
            time_manager: TimeManager::default(),
            generator_manager: GeneratorManager::new(),
        }
    }

    pub fn add_track_data(&mut self, track: TrackData) -> Result<(), crate::Error> {
        match self.tracks.entry(track.track_nr) {
            Entry::Occupied(_) => Err(crate::Error::Overwrite),
            Entry::Vacant(e) => {
                let id = SaveId::Track(track.track_nr);
                let mut decoded_track = MidiTrack::new();

                decoded_track.set_id(track.track_nr);

                // generators
                for (i, points) in track.gen_data.into_iter() {
                    let channel =
                        Generator::PointDefined(PointDefined::new(points, Interpolation::Step));
                    self.generator_manager
                        .add_generator_with_key(channel, id, i)
                        .unwrap();
                }
                // pitchbend
                if !track.pitch_bend.is_empty() {
                    let pitchbend = Generator::PointDefined(PointDefined::new(
                        track.pitch_bend,
                        Interpolation::Step,
                    ));
                    let pitchbend_id = self.generator_manager.add_generator(pitchbend, id).unwrap();
                    decoded_track.add_pitch_bend(pitchbend_id);
                }
                // channel after touch
                if !track.ch_after_touch.is_empty() {
                    let ch_after_touch = Generator::PointDefined(PointDefined::new(
                        track.ch_after_touch,
                        Interpolation::Step,
                    ));
                    let ch_after_touch_id = self
                        .generator_manager
                        .add_generator(ch_after_touch, id)
                        .unwrap();
                    decoded_track.add_ch_after_touch(ch_after_touch_id);
                }

                // TODO track.inst_name

                e.insert(Track::Midi(decoded_track));
                Ok(())
            }
        }
    }
}

impl Default for SongData {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Song> for SongData {
    fn from(song: &Song) -> Self {
        Self {
            name: song.name.clone(),
            tracks: song.tracks.clone(),
            time_manager: TIME_MANAGER.read().unwrap().clone(),
            generator_manager: GENRATOR_MANAGER.read().unwrap().clone(),
        }
    }
}

impl From<SongData> for Song {
    fn from(data: SongData) -> Self {
        *GENRATOR_MANAGER.write().unwrap() = data.generator_manager;
        *TIME_MANAGER.write().unwrap() = data.time_manager;
        Self {
            name: data.name,
            tracks: data.tracks,
        }
    }
}

struct TimeDecoder {
    midi_timeing: Timing,
    tempo_mus: Vec<(u32, u32)>,
    time_signatures: Vec<(u32, TimeSignature)>,
}

impl TimeDecoder {
    pub fn new(timing: Timing) -> Self {
        Self {
            midi_timeing: timing,
            tempo_mus: Vec::new(),
            time_signatures: Vec::new(),
        }
    }

    pub fn push_tempo(&mut self, tick: u32, value: u32) {
        self.tempo_mus.push((tick, value))
    }

    pub fn push_signature(&mut self, tick: u32, numerator: u8, denominator: u8) {
        self.time_signatures.push((
            tick,
            TimeSignature {
                beats_per_bar: numerator,
                beat_value: denominator,
                subdivision: None,
            },
        ))
    }

    pub fn validate(&mut self) -> Result<(), Box<dyn Error>> {
        self.tempo_mus.sort_by_key(|x| x.0);
        self.tempo_mus.dedup();
        for i in 0..(self.tempo_mus.len() - 1) {
            let t1 = self.tempo_mus[i];
            let t2 = self.tempo_mus[i + 1];
            if t1.0 == t2.0 && !t1.1 == t2.1 {
                return Err(Box::new(crate::Error::Parse));
            }
        }

        self.time_signatures.sort_by_key(|x| x.0);
        self.time_signatures.dedup();
        for i in 0..(self.time_signatures.len() - 1) {
            let t1 = &self.time_signatures[i];
            let t2 = &self.time_signatures[i + 1];
            if t1.0 == t2.0 && !(t1.1 == t2.1) {
                return Err(Box::new(crate::Error::Parse));
            }
        }
        Ok(())
    }

    pub fn decode_ticks(&self, tick: u32) -> TimeStamp {
        todo!()
    }
}

struct AlmostTrack {
    name: String,
    inst_name: String,
    number: u16,
    note_on: Vec<NoteOn>,
    note_off: Vec<NoteOff>,
    after_touch: Vec<AfterTouch>,
    ch_after_touch: Vec<ChAftertouch>,
    cc: Vec<ControlChange>,
    pitch_bend: Vec<PitchBend>,
}

impl AlmostTrack {
    pub fn new(number: u16) -> Self {
        Self {
            name: String::new(),
            inst_name: String::new(),
            number,
            ch_after_touch: Vec::new(),
            after_touch: Vec::new(),
            note_on: Vec::new(),
            note_off: Vec::new(),
            cc: Vec::new(),
            pitch_bend: Vec::new(),
        }
    }

    pub fn change_name(&mut self, name: &str) {
        self.name = name.to_string()
    }

    pub fn change_inst_name(&mut self, name: &str) {
        self.inst_name = name.to_string()
    }

    pub fn change_number(&mut self, number: u16) {
        self.number = number
    }

    pub fn push_note_on(&mut self, tick: u32, key: u7, vel: u7) {
        self.note_on.push(NoteOn {
            tick,
            key: key.as_int(),
            vel: vel.as_int(),
        })
    }

    pub fn push_note_off(&mut self, tick: u32, key: u7) {
        self.note_off.push(NoteOff {
            tick,
            key: key.as_int(),
        })
    }

    pub fn push_cc(&mut self, tick: u32, control: u7, val: u7) {
        self.cc.push(ControlChange {
            tick,
            control: control.as_int(),
            val: val.as_int(),
        })
    }

    pub fn push_pitch_bend(&mut self, tick: u32, val: midly::PitchBend) {
        self.pitch_bend.push(PitchBend { tick, val })
    }

    pub fn push_after_touch(&mut self, tick: u32, key: u7, vel: u7) {
        self.after_touch.push(AfterTouch {
            tick,
            key: key.as_int(),
            vel: vel.as_int(),
        })
    }

    pub fn push_ch_after_touch(&mut self, tick: u32, vel: u7) {
        self.ch_after_touch.push(ChAftertouch {
            tick,
            vel: vel.as_int(),
        })
    }
}

impl AlmostTrack {
    pub fn into_track(self, time_decoder: &TimeDecoder) -> TrackData {
        let mut notes = Vec::new();
        assert_eq!(
            self.note_off.len(),
            self.note_on.len(),
            "different count of note on and note off events"
        );
        // TODO AfterTouch
        for note_on in self.note_on.into_iter() {
            let mut index = None;
            for (i, note_off) in self.note_off.iter().enumerate() {
                if note_on.key == note_off.key {
                    index = Some(i);
                    break;
                }
            }
            notes.push(midi::Note {
                pitch: midi::Pitch::new(note_on.key).unwrap(),
                on: time_decoder.decode_ticks(note_on.tick),
                off: time_decoder.decode_ticks(self.note_off[index.unwrap()].tick),
                velocity: note_on.vel as f64 / 127.0,
            })
        }

        let mut pitch_bend = Vec::new();

        for p in self.pitch_bend {
            pitch_bend.push(
                AutomationPoint::new(
                    p.val.as_f64() / 2.0 + 0.5,
                    time_decoder.decode_ticks(p.tick),
                )
                .unwrap(),
            )
        }

        let mut gen_data = HashMap::<u8, Vec<AutomationPoint>>::new();

        for p in self.cc.into_iter() {
            let point =
                AutomationPoint::new(p.val as f64 / 127.0, time_decoder.decode_ticks(p.tick))
                    .unwrap();
            match gen_data.entry(p.control) {
                Entry::Occupied(e) => e.into_mut().push(point),
                Entry::Vacant(e) => {
                    e.insert(vec![point]);
                }
            }
        }

        let mut ch_after_touch = Vec::new();
        for p in self.ch_after_touch.into_iter() {
            ch_after_touch.push(
                AutomationPoint::new(p.vel as f64 / 127.0, time_decoder.decode_ticks(p.tick))
                    .unwrap(),
            )
        }

        TrackData {
            name: self.name,
            inst_name: self.inst_name,
            track_nr: self.number as u8,
            ch_after_touch,
            notes,
            gen_data,
            pitch_bend,
        }
    }
}

struct NoteOn {
    tick: u32,
    key: u8,
    vel: u8,
}

struct NoteOff {
    tick: u32,
    key: u8,
}

struct ControlChange {
    tick: u32,
    control: u8,
    val: u8,
}

struct PitchBend {
    tick: u32,
    val: midly::PitchBend,
}

struct AfterTouch {
    tick: u32,
    key: u8,
    vel: u8,
}

struct ChAftertouch {
    tick: u32,
    vel: u8,
}

#[derive(Debug)]
pub struct TrackData {
    pub name: String,
    pub inst_name: String,
    pub track_nr: u8,
    pub notes: Vec<midi::Note>,
    pub gen_data: HashMap<u8, Vec<AutomationPoint>>,
    pub pitch_bend: Vec<AutomationPoint>,
    pub ch_after_touch: Vec<AutomationPoint>,
}
