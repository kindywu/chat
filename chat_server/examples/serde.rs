use serde::{Deserialize, Serialize};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let chat_id: i64 = 10;
    let json = serde_json::to_string(&chat_id)?;
    println!("{json}");

    let output = Output { chat_id: 10 };
    let json = serde_json::to_string(&output)?;
    println!("{json}");

    let json = json!({
        "chat_id": 10,
    });
    println!("{json}");

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Output {
    chat_id: i64,
}
