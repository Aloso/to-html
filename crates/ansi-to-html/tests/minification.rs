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
                "res" => out.push_str("0"),
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

#[test]
fn ariadne() {
    let readable = "\
{{ red }}Error:{{ res }} Incompatible types
   {{ 8_246 }},{{ res }}{{ 8_246 }}-{{ res }}{{ 8_246 }}[{{ res }}<unknown>:2:9{{ 8_246 }}]{{ res }}
   {{ 8_246 }}|{{ res }}
 {{ 8_246 }}2 |{{ res }} {{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}\
 {{ 8_249 }} {{ res }}{{ 8_249 }}({{ res }}{{ 8_249 }}){{ res }}{{ 8_249 }} {{ res }}\
 {{ 8_249 }}={{ res }}{{ 8_249 }}>{{ res }}{{ 8_249 }} {{ res }}{{ cyan }}5{{ res }}\
 {{ 8_249 }},{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ cyan }}|{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ cyan }}`{{ res }}{{ cyan }}-{{ res }}{{ cyan }}-{{ res }} \
 This is of type Nat
 {{ 8_246 }}3 |{{ res }} {{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}{{ 8_249 }} {{ res }}\
 {{ 8_249 }} {{ res }}{{ 8_249 }}({{ res }}{{ 8_249 }}){{ res }}{{ 8_249 }} {{ res }}\
 {{ 8_249 }}={{ res }}{{ 8_249 }}>{{ res }}{{ 8_249 }} {{ res }}{{ blue }}\"{{ res }}\
 {{ blue }}5{{ res }}{{ blue }}\"{{ res }}{{ 8_249 }},{{ res }}
 {{ 8_240 }}  |{{ res }}           {{ blue }}^{{ res }}{{ blue }}|{{ res }}{{ blue }}^{{ res }}
 {{ 8_240 }}  |{{ res }}            {{ blue }}`{{ res }}{{ blue }}-{{ res }}{{ blue }}-{{ res }}\
 {{ blue }}-{{ res }} This is of type Str
{{ 8_246 }}---'{{ res }}\
    ";

    let styled = human_readable_to_ansi(readable);

    let converted = ansi_to_html::convert_escaped(&styled).unwrap();
    insta::assert_snapshot!(converted);
}
