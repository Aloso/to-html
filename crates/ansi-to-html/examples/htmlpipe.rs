use std::io::{self, Read};

use clap::Parser;

/// A small demo that converts ANSI stdin to HTML. Typical usage would be something like
///
/// $ echo -e 'Plain \e[1mBold' | cargo run -q --example pipe > output.html
///
/// $ firefox output.html
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    /// Skip escaping special HTML characters before conversion
    #[arg(long)]
    skip_escape: bool,
    /// Skip optimized the converted HTML
    #[arg(long)]
    skip_optimize: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our CLI args
    let Args {
        skip_escape,
        skip_optimize,
    } = Args::parse();

    // HTMLify our stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let htmlified = ansi_to_html::Converter::new()
        .skip_escape(skip_escape)
        .skip_optimize(skip_optimize)
        .convert(&input)?;

    // Wrapping the output in `<pre>` to preserve the whitespace
    println!("<pre>\n{htmlified}</pre>");
    Ok(())
}
