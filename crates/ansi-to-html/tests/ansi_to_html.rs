/// Converts human readable tokens to ANSI color codes
///
/// Some sample conversion:
/// - Reset: {{ res }}
/// - Blue foreground: {{ blue }}
/// - 8-bit code: {{ 8_<lookup_number> }}
fn human_readable_to_ansi(s: &str) -> String {
    let mut out = String::new();

    let mut without_left = s.split("{{ ");

    out.push_str(without_left.next().unwrap_or_default());

    for chunk in without_left {
        if let Some((inner, text)) = chunk.split_once(" }}") {
            // This is missing a lot of tokens. I just added enough to get the tests running
            out.push_str("\x1b[");
            match inner {
                // Control
                "res" => out.push('0'),
                // Style
                "bold" => out.push('1'),
                "underline" => out.push('4'),
                "underline_off" => out.push_str("24"),
                // Basic colors
                "blue" => out.push_str("34"),
                "cyan" => out.push_str("36"),
                "red" => out.push_str("31"),
                // 8-bit foreground colors
                "8_240" | "8_246" | "8_249" => {
                    let num = inner.strip_prefix("8_").unwrap();
                    out.push_str("38;5;");
                    out.push_str(num);
                }
                // Well this can also get false positives from pairs of {{ and }} that aren't for a
                // human readable token, but we control the test input, so it's not a big deal
                other => todo!("Missing arm for {other}"),
            }
            out.push('m');
            out.push_str(text);
        } else {
            out.push_str("{{ ");
            out.push_str(chunk);
        }
    }

    out
}

// `ariadne`, at least at the time of writing, has the bad habit of inserting a lot of redundant
// styling for the same run of text. This test ensures that the redundant styling gets correctly
// discarded while minifying. Issue: https://github.com/Aloso/to-html/issues/17
#[test]
fn ariadne() {
    // Without styling:
    // Error: Incompatible types
    //    ,-[<unknown>:2:10]
    //    |
    //  2 |     () => 5,
    //    |           |
    //    |           `-- This is of type Nat
    //  3 |     () => "5",
    //    |           ^|^
    //    |            `--- This is of type Str
    // ---'
    let readable = r#"
{{ red }}Error:{{ res }} Incompatible types
   {{ 8_246 }},{{ res }}{{ 8_246 }}-{{ res }}{{ 8_246 }}[{{ res }}<unknown>:2:9{{ 8_246 }}]{{ res }}
   {{ 8_246 }}|{{ res }}
 {{ 8_246 }}2 |{{ res }} {{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }}({{ res }}{{ 8_249 }}){{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }}={{ res }}{{ 8_249 }}>{{ res }}{{ 8_249 }} {{ res }}{{ cyan }}5{{ res }}{{ 8_249 }},{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ cyan }}|{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ cyan }}`{{ res }}{{ cyan }}-{{ res }}{{ cyan }}-{{ res }} This is of type Nat
 {{ 8_246 }}3 |{{ res }} {{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }}({{ res }}{{ 8_249 }}){{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }}={{ res }}{{ 8_249 }}>{{ res }}{{ 8_249 }} {{ res }}{{ blue }}"{{ res }}{{ blue }}5{{ res }}{{ blue }}"{{ res }}{{ 8_249 }},{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ blue }}^{{ res }}{{ blue }}|{{ res }}{{ blue }}^{{ res }}
 {{ 8_240 }}  |{{ res }}            {{ blue }}`{{ res }}{{ blue }}-{{ res }}{{ blue }}-{{ res }}{{ blue }}-{{ res }} This is of type Str
{{ 8_246 }}---'{{ res }}
    "#;

    let styled = human_readable_to_ansi(readable.trim());

    let converted = ansi_to_html::convert(&styled).unwrap();
    insta::assert_snapshot!(converted, @r###"
    <span style='color:var(--red,#a00)'>Error:</span> Incompatible types
       <span style='color:#949494'>,-[</span>&lt;unknown&gt;:2:9<span style='color:#949494'>]</span>
       <span style='color:#949494'>|</span>
     <span style='color:#949494'>2 |</span> <span style='color:#b2b2b2'>    () =&gt; </span><span style='color:var(--cyan,#0aa)'>5</span><span style='color:#b2b2b2'>,</span>
     <span style='color:#585858'>  |</span>           <span style='color:var(--cyan,#0aa)'>|</span>
     <span style='color:#585858'>  |</span>           <span style='color:var(--cyan,#0aa)'>`--</span> This is of type Nat
     <span style='color:#949494'>3 |</span> <span style='color:#b2b2b2'>    () =&gt; </span><span style='color:var(--blue,#00a)'>&quot;5&quot;</span><span style='color:#b2b2b2'>,</span>
     <span style='color:#585858'>  |</span>           <span style='color:var(--blue,#00a)'>^|^</span>
     <span style='color:#585858'>  |</span>            <span style='color:var(--blue,#00a)'>`---</span> This is of type Str
    <span style='color:#949494'>---&#39;</span>
    "###);
}

#[test]
fn semicolon_before_terminator() {
    let converted = ansi_to_html::convert("\x1b[31;mRed\x1b[0;m Plain").unwrap();
    insta::assert_snapshot!(converted, @"<span style='color:var(--red,#a00)'>Red</span> Plain");
}

#[test]
fn minifier_discards_useless_styles() {
    let input_to_expected = [
        ("{{ bold }}{{ res }}{{ bold }}", ""),
        ("Plain{{ blue }}{{ bold }}", "Plain"),
        (
            "{{ bold }}Bold{{ bold }}{{ blue }}{{ res }}{{ bold }} ... still bold",
            "<b>Bold ... still bold</b>",
        ),
        ("Plain{{ bold }}Bold{{ red }}", "Plain<b>Bold</b>"),
        ("{{ bold }}Bold{{ blue }}", "<b>Bold</b>"),
        (
            "{{ underline }}Underline{{ bold }}{{ underline_off }}Bold",
            "<u>Underline</u><b>Bold</b>",
        ),
    ];

    for (input, expected) in input_to_expected {
        let styled = human_readable_to_ansi(input);
        let converted = ansi_to_html::Converter::new()
            .skip_optimize(true)
            .convert(&styled)
            .unwrap();
        assert_eq!(
            converted, expected,
            "Styles without any text should be ommitted by the minifier"
        );
    }
}

// TODO: switch fuzz test over to a prop test after it stops picking up bugs
