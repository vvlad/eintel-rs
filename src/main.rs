mod app;
mod chat;
mod config;
mod errors;
mod events;
mod intel;
mod notifications;
mod universe;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate encoding;
extern crate fern;
extern crate memmap;
extern crate notify;
extern crate regex;

use errors::*;
use fern::colors::ColoredLevelConfig;
quick_main!(run);

fn run() -> Result<()> {
    let colors = ColoredLevelConfig::default();

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[ {} ] {} > {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    let conf = config::Config::default()?
        .player("Derzerek")
        .player("Yolla")
        .player("Inge Inkura")
        .channel("GotG Home Intel")
        .channel("Derzerek");

    info!("Starting the app");
    app::run(conf)?;
    Ok(())
}
