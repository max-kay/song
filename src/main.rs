use std::path::Path;

use song::io;

fn main() {
    let path = Path::new("midi_files/seven8.mid");
    let song_data = io::parse_midi_file(path).unwrap();
    let string = ron::ser::to_string_pretty(&song_data, Default::default()).unwrap();
    println!("{}", string)
}
