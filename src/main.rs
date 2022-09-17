use song::{
    instr::{drums::DrumsBuilder, synth::SynthBuilder},
    Song,
};

fn main() {
    let mut song = Song::from_midi("midi_files/short_test.mid").unwrap();
    let synth = SynthBuilder::from_path("instr/boring.ron").unwrap();
    for track in song.mut_midi_tracks() {
        match track.get_name() {
            "Classic Electric Piano" => track.add_synth(synth.clone()),
            "Fingerstyle Bass" => track.add_synth(synth.clone()),
            "Bluebird" => {
                track
                    .add_drums(DrumsBuilder::from_path("instr/drums.ron").unwrap())
                    .unwrap();
            }
            _ => (),
        }
    }
    song.get_wave().dirty_play();
    song.save_to("songs/short_test.ron").unwrap();
}
