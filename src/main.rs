use clap::StructOpt;
use color_eyre::eyre::Result;
use mod_use::mod_use;

mod_use![cli, api, model, config, util];

fn main() -> Result<()> {
    color_eyre::install()?;
    init_logger()?;

    Opt::parse().handle()
}
