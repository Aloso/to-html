#![no_main]

use ansi_to_html::Converter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let Ok(opt) = ansi_to_html::convert(data) else {
        return;
    };
    let no_opt = Converter::new().skip_optimize(true).convert(data).unwrap();
    assert_eq!(
        opt.len(),
        no_opt.len(),
        "\nOptimized:\n{opt}\nUnoptimized:\n{no_opt}"
    );
});
