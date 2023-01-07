use std::{
    fs::{self, File},
    path::Path,
};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde::{Deserialize as De, Serialize as Ser};

use crate::{API, BIN_NAME, CONFIG_PATH};

#[derive(Ser, De, Debug)]
pub struct Config {
    pub auth: Auth,
}

impl Config {
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let content = String::from_utf8(fs::read(path)?)?;
            Ok(ron::from_str(&content)?)
        } else {
            let err = eyre!("For more help, see https://github.com/George-Miao/shot#help")
                .wrap_err(format!(
                    "Use `{BIN_NAME} auth <account_id> <token>` or manually edit \
                     `~/{CONFIG_PATH}`."
                ))
                .wrap_err("You haven't config your authentication info yet");
            Err(err)
        }
    }

    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }

    pub fn into_api(self) -> Result<API> {
        API::new(self.auth)
    }

    pub fn as_api(&self) -> Result<API> {
        API::new(self.auth.clone())
    }

    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let mut file =
            File::create(path).wrap_err_with(|| format!("Bad path {}", path.display()))?;
        ron::ser::to_writer_pretty(&mut file, self, Default::default())
            .wrap_err_with(|| format!("Unable to write config file to {}", path.display()))?;
        Ok(())
    }
}

#[derive(Ser, De, Debug, Parser, Clone, PartialEq, Eq)]
#[group(skip)]
pub struct Auth {
    pub account_id: String,
    pub token: String,
}

impl<S1, S2> From<(S1, S2)> for Auth
where
    S1: Into<String>,
    S2: Into<String>,
{
    fn from(val: (S1, S2)) -> Auth {
        let (account_id, token) = (val.0.into(), val.1.into());
        Auth { account_id, token }
    }
}
