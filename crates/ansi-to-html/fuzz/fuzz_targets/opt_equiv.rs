#![no_main]

use ansi_to_html::test_utils;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|ansi_text: &str| {
    test_utils::assert_opt_equiv_to_no_opt(ansi_text);
});
