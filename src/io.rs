use self::data::{MidiTrackBuilder, SongBuilder};
use crate::{time::ClockTick, tracks::midi, utils::XYPairs, wave::Wave, globals::SAMPLE_RATE};
use hound::{SampleFormat, WavSpec};
use itertools::Itertools;
use midly::{Format, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fs,
    path::Path,
};

pub mod data;

pub fn read_wav(path: impl AsRef<Path>) -> Result<Wave, Box<dyn std::error::Error>> {
    let reader = hound::WavReader::open(path)?;
    let WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format,
    } = reader.spec();

    if channels != 2 {
        Err(crate::Error::WavRead)?
    };

    if sample_rate as usize != SAMPLE_RATE {
        Err(crate::Error::WavRead)?
    };

    match sample_format {
        SampleFormat::Float => match bits_per_sample {
            0..=32 => {
                todo!()
            }
            33..=64 => {
                todo!()
            }
            _ => Err(crate::Error::WavRead)?,
        },
        SampleFormat::Int => match bits_per_sample {
            0..=8 => {
                todo!()
            }
            9..=16 => {
                let mut right: Vec<i16> = Vec::with_capacity(reader.len() as usize / 2);
                let mut left: Vec<i16> = Vec::with_capacity(reader.len() as usize / 2);
                for mut chunk in &reader
                    .into_samples::<i16>()
                    .map(|s| s.expect("error in reading wav"))
                    .collect::<Vec<i16>>()
                    .into_iter()
                    .chunks(2)
                {
                    left.push(chunk.next().unwrap());
                    right.push(chunk.next().unwrap());
                }
                Ok(Wave::from_vecs(
                    right
                        .into_iter()
                        .map(|x| x as f32 / i16::MAX as f32)
                        .collect(),
                    left.into_iter()
                        .map(|x| x as f32 / i16::MAX as f32)
                        .collect(),
                ))
            }
            17..=32 => {
                todo!()
            }
            33..=64 => {
                todo!()
            }
            _ => Err(crate::Error::WavRead)?,
        },
    }
}

// what midi calls ... I call ...
// clock -> tick
// quater -> beat
// tempo -> mus_per_beat
// 32nd per quater -> n32nd_per_beat

pub fn parse_midi_file(path: impl AsRef<Path>) -> Result<SongBuilder, Box<dyn Error>> {
    let bytes = fs::read(&path)?;
    let smf = Smf::parse(&bytes)?;
    let mut time_decoder = TimeDecoder::new(smf.header.timing);
    if smf.header.format == Format::Sequential {
        todo!()
    }
    let mut almost_tracks = Vec::new();
    for (i, track) in smf.tracks.iter().enumerate() {
        almost_tracks.push(parse_midi_track(
            track,
            &mut time_decoder,
            i.try_into().unwrap(),
        )?)
    }
    let mut decoded = SongBuilder::new();
    for track in almost_tracks {
        decoded
            .add_track_data(track.into_track()?)
            .expect("time decoding error in track");
    }

    decoded.set_time_manager(time_decoder.into());
    Ok(decoded)
}

fn parse_midi_track(
    track: &[TrackEvent],
    time_decoder: &mut TimeDecoder,
    track_index: u16,
) -> Result<AlmostTrack, Box<dyn Error>> {
    let mut data = AlmostTrack::new(track_index);
    let mut current_ticks = 0;
    for event in track {
        current_ticks += event.delta.as_int();
        match event.kind {
            TrackEventKind::Meta(msg) => {
                decode_meta_msg(msg, &mut data, time_decoder, current_ticks)?
            }
            TrackEventKind::Midi {
                channel: _,
                message,
            } => match message {
                MidiMessage::NoteOn { key, vel } => {
                    data.push_note_on(current_ticks, key.as_int(), vel.as_int())
                }
                MidiMessage::NoteOff { key, vel: _ } => {
                    data.push_note_off(current_ticks, key.as_int())
                }
                MidiMessage::Controller { controller, value } => {
                    data.push_cc(current_ticks, controller.as_int(), value.as_int())
                }
                MidiMessage::PitchBend { bend } => data.push_pitch_bend(current_ticks, bend),
                MidiMessage::Aftertouch { key, vel } => {
                    data.push_after_touch(current_ticks, key.as_int(), vel.as_int())
                }
                MidiMessage::ChannelAftertouch { vel } => {
                    data.push_ch_after_touch(current_ticks, vel.as_int())
                }
                MidiMessage::ProgramChange { program: _ } => (),
            },
            TrackEventKind::SysEx(_) => (),
            TrackEventKind::Escape(_) => (),
        }
    }
    Ok(data)
}

fn decode_meta_msg(
    msg: midly::MetaMessage,
    data: &mut AlmostTrack,
    time_decoder: &mut TimeDecoder,
    current_ticks: u32,
) -> Result<(), Box<dyn Error>> {
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
        Tempo(tempo) => time_decoder
            .mus_per_beat(current_ticks, tempo.as_int())
            .expect("failed to decode tempo msg"),
        TimeSignature(num, lb_den, _metronome, n32nd_per_beat) => {
            time_decoder.push_signature(current_ticks, num, lb_den, n32nd_per_beat)?
        }
        EndOfTrack => (),
        KeySignature(_, _) => (),
        MidiChannel(_) => (),
        SmpteOffset(a) => println!("ignored smpte offset: {:?}", a),
        Text(txt) => println!(
            "ignored text meta message: {:?}",
            String::from_utf8(txt.to_vec())
        ),
        Copyright(txt) => println!(
            "ignored text meta message:{:?}",
            String::from_utf8(txt.to_vec())
        ),
        _ => println!("ignored {:?}", msg),
    }
    Ok(())
}

// ticks_per_quarter =  <PPQ from the header>
// µs_per_quarter =     <Tempo in latest Set Tempo event>
// µs_per_tick =        µs_per_quarter / ticks_per_quarter
// seconds_per_tick =   µs_per_tick / 1.000.000
// seconds =            ticks * seconds_per_tick

pub(crate) struct TimeDecoder {
    pub midi_timeing: Timing,
    pub mus_per_beat: XYPairs<u32, u32>,
    pub time_signatures: XYPairs<u32, MidiSig>,
}

#[derive(Debug, Clone, Copy)]
pub struct MidiSig {
    beats_per_bar: u8,
    beat_value: u8,
    n32_p_beat: u8,
}

impl TimeDecoder {
    pub fn new(timing: Timing) -> Self {
        Self {
            midi_timeing: timing,
            mus_per_beat: Default::default(),
            time_signatures: Default::default(),
        }
    }

    pub fn mus_per_beat(&mut self, tick: u32, value: u32) -> Result<(), Box<dyn Error>> {
        self.mus_per_beat
            .push(tick, value)
            .map_err(|_| Box::new(crate::Error::Parse) as Box<dyn Error>)
    }

    pub fn push_signature(
        &mut self,
        tick: u32,
        numerator: u8,
        lb_denominator: u8,
        n32_p_beat: u8,
    ) -> Result<(), Box<dyn Error>> {
        self.time_signatures
            .push(
                tick,
                MidiSig {
                    beats_per_bar: numerator,
                    beat_value: 2_u8.pow(lb_denominator.into()),
                    n32_p_beat,
                },
            )
            .map_err(|_| Box::new(crate::Error::Parse) as Box<dyn Error>)
    }

    pub fn convert_mus_beat_to_s_tick(&self, mus: &u32) -> f32 {
        match self.midi_timeing {
            Timing::Metrical(tpb) => *mus as f32 / (1_000_000.0 * tpb.as_int() as f32),
            Timing::Timecode(_fps, _sfpf) => todo!(),
        }
    }
}

#[derive(Debug)]
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

    pub fn push_note_on(&mut self, tick: u32, key: u8, vel: u8) {
        self.note_on.push(NoteOn { tick, key, vel })
    }

    pub fn push_note_off(&mut self, tick: u32, key: u8) {
        self.note_off.push(NoteOff { tick, key })
    }

    pub fn push_cc(&mut self, tick: u32, control: u8, val: u8) {
        self.cc.push(ControlChange { tick, control, val })
    }

    pub fn push_pitch_bend(&mut self, tick: u32, val: midly::PitchBend) {
        self.pitch_bend.push(PitchBend { tick, val })
    }

    pub fn push_after_touch(&mut self, tick: u32, key: u8, vel: u8) {
        self.after_touch.push(AfterTouch { tick, key, vel })
    }

    pub fn push_ch_after_touch(&mut self, tick: u32, vel: u8) {
        self.ch_after_touch.push(ChAftertouch { tick, vel })
    }
}

impl AlmostTrack {
    pub fn into_track(mut self) -> Result<MidiTrackBuilder, Box<dyn Error>> {
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
                on: ClockTick::new(note_on.tick),
                off: ClockTick::new(self.note_off.remove(index.unwrap()).tick),
                velocity: note_on.vel as f32 / 127.0,
            })
        }

        let mut pitch_bend = XYPairs::new();

        for p in self.pitch_bend {
            pitch_bend.push(ClockTick::new(p.tick), p.val.as_f32() / 2.0 + 0.5)?
        }

        let mut gen_data = HashMap::<u8, XYPairs<_, _>>::new();

        for p in self.cc.into_iter() {
            let val = p.val as f32 / 127.0;
            let tick = ClockTick::new(p.tick);
            match gen_data.entry(p.control) {
                Entry::Occupied(e) => e
                    .into_mut()
                    .push(tick, val)
                    .map_err(|_| crate::Error::Parse)?,
                Entry::Vacant(e) => {
                    e.insert(XYPairs::from_point(tick, val));
                }
            }
        }

        let mut ch_after_touch = XYPairs::new();
        for p in self.ch_after_touch.into_iter() {
            ch_after_touch.push(ClockTick::new(p.tick), p.vel as f32 / 127.0)?
        }

        Ok(MidiTrackBuilder {
            name: self.name,
            inst_name: self.inst_name,
            track_nr: self.number as u8,
            _ch_after_touch: ch_after_touch,
            notes,
            gen_data,
            pitch_bend,
        })
    }
}

#[derive(Debug)]
struct NoteOn {
    tick: u32,
    key: u8,
    vel: u8,
}

#[derive(Debug)]
struct NoteOff {
    tick: u32,
    key: u8,
}

#[derive(Debug)]
struct ControlChange {
    tick: u32,
    control: u8,
    val: u8,
}

#[derive(Debug)]
struct PitchBend {
    tick: u32,
    val: midly::PitchBend,
}

#[derive(Debug)]
struct AfterTouch {
    tick: u32,
    key: u8,
    vel: u8,
}

#[derive(Debug)]
struct ChAftertouch {
    tick: u32,
    vel: u8,
}
