use std::error::Error;

pub mod ansi;
pub mod color;
pub mod esc;
pub mod html;

pub use esc::Esc;

pub fn to_html(ansi_string: &str) -> Result<String, Box<dyn Error>> {
    let input = Esc(ansi_string).to_string();
    let stdout = html::ansi_to_html(&input)?;
    Ok(stdout)
}
