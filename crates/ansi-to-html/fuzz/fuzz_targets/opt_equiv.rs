#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|ansi_text: &str| {
    html_interpreter::assert_opt_equiv_to_no_opt(ansi_text);
});
