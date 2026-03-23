//! Feishu approval API v4 CLI. Env: repo `SKILL.md`; AI: `docs/AI.md`. Loads `.env` from cwd on startup.

use anyhow::Result;

mod cli;

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    cli::run()
}
