use song::{ Song, instr::synth::SynthBuilder};

fn main() {
    let mut song = Song::from_path("songs/7eight.ron").unwrap();
    let synth = SynthBuilder::from_path("instr/not_so_boring.ron").unwrap();
    
    for track in song.mut_midi_tracks() {
        track.add_synth(synth.clone());
    }
    
    song.get_wave().save("out/wave.wav");
}
