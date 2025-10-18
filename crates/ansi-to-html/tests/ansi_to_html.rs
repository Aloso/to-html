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
                "inv" => out.push('7'),
                "inv_off" => out.push_str("27"),
                // Styles
                "underline" => out.push('4'),
                "double_underline" => out.push_str("21"),
                "underline_off" => out.push_str("24"),
                "overline" => out.push_str("53"),
                "overline_off" => out.push_str("55"),
                // Basic colors
                "blue" => out.push_str("34"),
                "cyan" => out.push_str("36"),
                "red" => out.push_str("31"),
                "green" => out.push_str("32"),
                // 8-bit foreground colors
                code if code.starts_with("8_") => {
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

fn human_readable_to_html(readable: &str) -> String {
    let styled = human_readable_to_ansi(readable);
    ansi_to_html::convert(&styled).unwrap()
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

    let converted = human_readable_to_html(readable.trim());

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
fn underlines() {
    let readable = "{{ underline }}Single{{ res }} {{ double_underline }}Double";
    let ansi_text = human_readable_to_ansi(readable);
    let opt = ansi_to_html::convert(&ansi_text).unwrap();
    let no_opt = ansi_to_html::Converter::new()
        .skip_optimize(true)
        .convert(&ansi_text)
        .unwrap();
    assert_eq!(
        opt, no_opt,
        "Optimized and unoptimized text should be equivalent in this case"
    );
    insta::assert_snapshot!(
        no_opt,
        @"<u>Single</u> <u style='text-decoration-style:double'>Double</u>"
    );
}

#[test]
fn ansi_8bit_specification_of_4bit_color() {
    let readable = r#"
The first sixteen colors in the 8-bit palette are de facto standardized as the old 4-bit palette:
{{ 8_0 }}black{{ res }}
{{ 8_1 }}red{{ res }}
{{ 8_2 }}green{{ res }}
{{ 8_3 }}yellow{{ res }}
{{ 8_4 }}blue{{ res }}
{{ 8_5 }}magenta{{ res }}
{{ 8_6 }}cyan{{ res }}
{{ 8_7 }}white{{ res }}
Where the bright colors, too, are bright in the overlap of the 8-bit and 4-bit palettes:
{{ 8_8 }}bright black{{ res }}
{{ 8_9 }}bright red{{ res }}
{{ 8_10 }}bright green{{ res }}
{{ 8_11 }}bright yellow{{ res }}
{{ 8_12 }}bright blue{{ res }}
{{ 8_13 }}bright magenta{{ res }}
{{ 8_14 }}bright cyan{{ res }}
{{ 8_15 }}bright white{{ res }}
    "#;

    let converted = human_readable_to_html(readable.trim());
    insta::assert_snapshot!(converted, @r"
    The first sixteen colors in the 8-bit palette are de facto standardized as the old 4-bit palette:
    <span style='color:var(--black,#000)'>black</span>
    <span style='color:var(--red,#a00)'>red</span>
    <span style='color:var(--green,#0a0)'>green</span>
    <span style='color:var(--yellow,#a60)'>yellow</span>
    <span style='color:var(--blue,#00a)'>blue</span>
    <span style='color:var(--magenta,#a0a)'>magenta</span>
    <span style='color:var(--cyan,#0aa)'>cyan</span>
    <span style='color:var(--white,#aaa)'>white</span>
    Where the bright colors, too, are bright in the overlap of the 8-bit and 4-bit palettes:
    <span style='color:var(--bright-black,#555)'>bright black</span>
    <span style='color:var(--bright-red,#f55)'>bright red</span>
    <span style='color:var(--bright-green,#5f5)'>bright green</span>
    <span style='color:var(--bright-yellow,#ff5)'>bright yellow</span>
    <span style='color:var(--bright-blue,#55f)'>bright blue</span>
    <span style='color:var(--bright-magenta,#f5f)'>bright magenta</span>
    <span style='color:var(--bright-cyan,#5ff)'>bright cyan</span>
    <span style='color:var(--bright-white,#fff)'>bright white</span>
    ");
}

#[test]
fn invert_ansi_code() {
    let readable = r#"
Useless codes are minified away:
{{ inv }}{{ inv_off }}{{ res }}
No existing color will set fg and bg:
{{ inv }}Black fg white bg{{ res }}
Multiple inverts is a noop:
{{ inv }}{{ inv }}Still white fg black bg{{ res }}
Invert works with custom colors:
{{ red }}Red on black {{ inv }}Black on red{{ res }}
Invert off does nothing by itself:
{{ inv_off }}Plain
Invert off disables invert:
{{ inv }}Black on white {{ inv_off }}White on black{{ res }}
Multiple invert offs count as one:
{{ inv }}Inverted {{ inv_off }}{{ inv_off }}Non-Inverted{{ res }}
Setting FG color while inverted actually sets BG
{{ red }}Red fg{{ inv }}Red bg{{ green }}Green bg{{ inv_off }}Green fg
    "#;

    let converted = human_readable_to_html(readable.trim());
    insta::assert_snapshot!(converted, @r"
    Useless codes are minified away:

    No existing color will set fg and bg:
    <span style='color:var(--black,#000);background:var(--bright-white,#fff)'>Black fg white bg</span>
    Multiple inverts is a noop:
    <span style='color:var(--black,#000);background:var(--bright-white,#fff)'><span style='color:var(--black,#000);background:var(--bright-white,#fff)'>Still white fg black bg</span></span>
    Invert works with custom colors:
    <span style='color:var(--red,#a00)'>Red on black <span style='color:var(--black,#000);background:var(--red,#a00)'>Black on red</span></span>
    Invert off does nothing by itself:
    Plain
    Invert off disables invert:
    <span style='color:var(--black,#000);background:var(--bright-white,#fff)'>Black on white </span>White on black
    Multiple invert offs count as one:
    <span style='color:var(--black,#000);background:var(--bright-white,#fff)'>Inverted </span>Non-Inverted
    Setting FG color while inverted actually sets BG
    <span style='color:var(--red,#a00)'>Red fg<span style='color:var(--black,#000);background:var(--red,#a00)'>Red bg<span style='background:var(--green,#0a0)'>Green bg</span></span><span style='color:var(--green,#0a0)'>Green fg</span></span>
    ");
}

#[test]
fn overline() {
    let readable = "{{ overline }}over {{ underline }}and under{{ underline_off }} just over\
                    {{ overline_off }} plain";
    let ansi_text = human_readable_to_ansi(readable);
    let converted = ansi_to_html::convert(&ansi_text).unwrap();
    insta::assert_snapshot!(
        converted,
        @"<u style='text-decoration:overline'>over <u>and under</u> just over</u> plain"
    );
}

#[test]
fn hyperlink() {
    let input = "Finished \
        \x1b]8;;https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles\
        \x1b\\`dev` profile [unoptimized + debuginfo]\x1b]8;;\x1b\\ \
        target(s) in 0.04s";
    let converted = ansi_to_html::convert(input).unwrap();
    insta::assert_snapshot!(
        converted,
        @"Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s"
    );
}
