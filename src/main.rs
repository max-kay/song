use song::{instr::Synthesizer, tracks::midi, Song};

fn main() {
    let mut song = Song::new("my Song".to_string());
    let mut track = midi::MidiTrack::new();
}
