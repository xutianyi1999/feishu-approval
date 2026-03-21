//! Feishu workflow approval CLI (Open API v4). See `skills/feishu-approval/SKILL.md` for env vars; loads `.env` from the working directory on startup.

use anyhow::Result;

mod cli;

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    cli::run()
}
