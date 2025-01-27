use std::{hint::black_box, io::Read, time::Duration};

use ansi_to_html::Converter;
use divan::{bench, counter::BytesCount, Bencher, Divan};
use flate2::bufread::GzDecoder;

fn main() {
    Divan::default()
        .min_time(Duration::from_millis(500))
        .config_with_args()
        .main();
}

static COMPRESSED_TERMINAL_SESSION: &[u8] = include_bytes!("../assets/terminal_session.gz");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Input {
    AnsiHeavy,
    PlainText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaybeEsc {
    Esc,
    SkipEsc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaybeOpt {
    Opt,
    SkipOpt,
}

#[bench(args = [
    (Input::AnsiHeavy, MaybeEsc::Esc, MaybeOpt::Opt),
    (Input::AnsiHeavy, MaybeEsc::Esc, MaybeOpt::SkipOpt),
    (Input::AnsiHeavy, MaybeEsc::SkipEsc, MaybeOpt::Opt),
    (Input::AnsiHeavy, MaybeEsc::SkipEsc, MaybeOpt::SkipOpt),
    (Input::PlainText, MaybeEsc::Esc, MaybeOpt::Opt),
    (Input::PlainText, MaybeEsc::Esc, MaybeOpt::SkipOpt),
    (Input::PlainText, MaybeEsc::SkipEsc, MaybeOpt::Opt),
    (Input::PlainText, MaybeEsc::SkipEsc, MaybeOpt::SkipOpt),
])]
fn convert(bencher: Bencher, (input, esc, opt): (Input, MaybeEsc, MaybeOpt)) {
    let mut decoder = GzDecoder::new(COMPRESSED_TERMINAL_SESSION);
    let mut terminal_session = String::new();
    decoder.read_to_string(&mut terminal_session).unwrap();

    if input == Input::PlainText {
        // Replace the start of all ansi escape sequences with a benign character to "strip" all of
        // the ansi codes (well replace them with non-ansi garbage)
        terminal_session = terminal_session.replace('\u{1b}', "~");
    }

    let bytes_counter = BytesCount::of_str(&terminal_session);

    let converter = Converter::new()
        .skip_escape(esc == MaybeEsc::SkipEsc)
        .skip_optimize(opt == MaybeOpt::SkipOpt);

    bencher
        .counter(bytes_counter)
        .bench(|| converter.convert(black_box(&terminal_session)).unwrap());
}
