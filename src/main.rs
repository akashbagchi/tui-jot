mod app;
mod config;
mod core;
mod input;
mod ui;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = config::Config::load()?;
    let mut app = App::new(config)?;

    app.run().await
}
