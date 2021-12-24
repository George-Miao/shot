use clap::Parser;
use color_eyre::eyre::{Context, Result};

mod_use::mod_use![cli, api, model, config, util];

fn main() -> Result<()> {
    init().wrap_err("Internal error")?;
    Opt::parse().handle()
}
