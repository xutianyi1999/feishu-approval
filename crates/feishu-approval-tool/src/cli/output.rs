use anyhow::Result;
use serde_json::Value;

pub fn print_body(text: &str, pretty: bool) -> Result<()> {
    if pretty {
        if let Ok(v) = serde_json::from_str::<Value>(text) {
            println!("{}", serde_json::to_string_pretty(&v)?);
            return Ok(());
        }
    }
    print!("{text}");
    if !text.ends_with('\n') {
        println!();
    }
    Ok(())
}
