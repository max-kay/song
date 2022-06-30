use crate::consts::SAMPLE_RATE;
use crate::utils;
use std::fs::File;
use std::path::Path;
use wav::{self, WAV_FORMAT_PCM};

pub fn save_m_i16_wav(track: Vec<f64>, path: &Path) -> std::io::Result<()> {
    let header = wav::Header::new(WAV_FORMAT_PCM, 1, SAMPLE_RATE, 16);
    let track = wav::BitDepth::Sixteen(
        track
            .into_iter()
            .map(|x| (x * (i16::MAX as f64) / 4.0) as i16)
            .collect(),
    );
    let mut out_file = File::create(path).expect("Error while making file!");
    wav::write(header, &track, &mut out_file)
}

pub fn easy_save(track: Vec<f64>, path: &Path) {
    let track = utils::normalize(&track);
    save_m_i16_wav(track, path).expect("Error in easy_save")
}
