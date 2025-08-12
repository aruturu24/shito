mod app;
mod db;
mod dice;
mod models;

use anyhow::Result;

fn main() -> Result<()> {
    let db = db::Db::open_or_create("shito.sqlite3")?;
    let mut app = app::App::new(db)?;
    app.run()?;
    Ok(())
}
