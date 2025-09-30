use crate::{assert_opt_equiv_to_no_opt, interpret_html};

#[test]
fn sanity() {
    let ansi_text = "\x1b[1mBold\x1b[31mRed and Bold";
    let htmlified = ansi_to_html::convert(ansi_text).unwrap();
    insta::assert_debug_snapshot!(interpret_html(&htmlified), @r#"
    [
        StylizedText {
            styles: Styles {
                bold: true,
                italic: false,
                underlined: None,
                crossed_out: false,
                spans: {},
            },
            text: "Bold",
        },
        StylizedText {
            styles: Styles {
                bold: true,
                italic: false,
                underlined: None,
                crossed_out: false,
                spans: {
                    [
                        Attr {
                            name: "style",
                            value: "color:var(--red,#a00)",
                        },
                    ],
                },
            },
            text: "Red and Bold",
        },
    ]
    "#);
}

#[test]
fn overlapping_colors_sanity() {
    // Input: blue -> red -> "Red" -> red -> " Still Red"
    let ansi_text = "\x1b[34;31mRed\x1b[31m Still Red";
    assert_opt_equiv_to_no_opt(ansi_text);
    let htmlified = ansi_to_html::convert(ansi_text).unwrap();
    insta::assert_snapshot!(
        htmlified,
        @"<span style='color:var(--blue,#00a)'><span style='color:var(--red,#a00)'>Red Still Red</span></span>"
    );
}

#[test]
fn can_apply_already_applied_color() {
    // Input: red -> "Red" -> blue -> red -> " Still Red"
    let ansi_text = "\x1b[31mRed\x1b[34;31m Still Red";
    assert_opt_equiv_to_no_opt(ansi_text);
    let htmlified = ansi_to_html::convert(ansi_text).unwrap();
    insta::assert_snapshot!(
        htmlified,
        @"<span style='color:var(--red,#a00)'>Red Still Red</span>"
    );
}

/// Previously when active styles were removed from the stack it would accidentally reapply
/// some of the active styles in the reverse order
#[test]
fn removing_style_keeps_correct_order() {
    // Input: underline -> blue -> red -> underline off -> "Red" -> red -> " Still Red"
    let ansi_text = "\x1b[4;34;31;24mRed\x1b[31m Still Red";
    assert_opt_equiv_to_no_opt(ansi_text);
    let htmlified = ansi_to_html::convert(ansi_text).unwrap();
    insta::assert_snapshot!(
        htmlified,
        @"<u><span style='color:var(--blue,#00a)'></span></u><span style='color:var(--blue,#00a)'><span style='color:var(--red,#a00)'>Red Still Red</span></span>"
    );
}
