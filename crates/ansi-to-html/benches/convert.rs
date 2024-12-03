use std::{hint::black_box, io::Read, time::Duration};

use divan::{bench, counter::BytesCount, Bencher, Divan};
use flate2::bufread::GzDecoder;

fn main() {
    Divan::default()
        .min_time(Duration::from_millis(500))
        .config_with_args()
        .main();
}

static COMPRESSED_TERMINAL_SESSION: &[u8] = include_bytes!("../assets/terminal_session.gz");

#[bench]
fn convert(bencher: Bencher) {
    let mut decoder = GzDecoder::new(COMPRESSED_TERMINAL_SESSION);
    let mut terminal_session = String::new();
    decoder.read_to_string(&mut terminal_session).unwrap();

    let bytes_counter = BytesCount::of_str(&terminal_session);
    bencher
        .counter(bytes_counter)
        .bench(|| ansi_to_html::convert(black_box(&terminal_session)).unwrap());
}
