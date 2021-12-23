use std::{path::PathBuf, str::FromStr};

use arboard::Clipboard;
use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
    Result,
};
use home::home_dir;
use image::{io::Reader, GenericImageView};
use log::{error, info};

use crate::{image_data_to_png, image_name, Auth, Config};

pub const CONFIG_PATH: &str = ".config/shot.ron";

pub const BIN_NAME: &str = clap::crate_name!();

#[derive(Parser, Debug)]
pub struct Opt {
    #[clap(subcommand)]
    cmd: Option<Cmd>,
    #[clap(flatten)]
    flag: Flag,
}

#[derive(Parser, Debug)]
pub struct Flag {
    #[clap(short, long)]
    /// Preview the command without perform any actions
    dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Auth of Cloudflare API.
    /// Currently only supports account_id + token pair
    Auth {
        #[clap(flatten)]
        auth: Auth,
    },
    /// Upload image in clipboard to Cloudflare Image
    Paste {
        #[clap(short = 'n', long)]
        /// Filename of the image, default to upload time in rfc3999 format (e.g. 2021-12-20T01:01:01Z.png)
        file_name: Option<String>,

        #[clap(short, long)]
        /// User modifyable key-value store that binds to image. Takes multiple value
        /// Format: $KEY=$VALUE
        metadata: Vec<KV>,
    },
    /// Encode local images to PNG and upload to Cloudflare Images.
    /// For all supported image format,
    /// see `https://docs.rs/image/latest/image/codecs/index.html#supported-formats`.
    Upload {
        /// Path of image to be uploaded
        file_path: PathBuf,

        #[clap(short = 'n', long)]
        /// Filename of the image, default to local file name
        file_name: Option<String>,

        #[clap(short, long)]
        /// User modifyable key-value store that binds to image. Takes multiple value
        /// Format: $KEY=$VALUE
        metadata: Vec<KV>,
    },
}

impl Default for Cmd {
    fn default() -> Self {
        Self::Paste {
            file_name: None,
            metadata: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KV {
    k: String,
    v: String,
}

impl FromStr for KV {
    type Err = color_eyre::eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splitted = s.splitn(2, '=');
        let k = splitted
            .next()
            .wrap_err("Parse failed. Format: K=V")?
            .to_string();
        let v = splitted
            .next()
            .wrap_err("Parse failed. Format: K=V")?
            .to_string();
        Ok(Self { k, v })
    }
}

impl Opt {
    pub fn handle(self) -> Result<()> {
        let config_path = home_dir()
            .wrap_err("Cannot determine home directory")?
            .join(CONFIG_PATH);
        let flag = self.flag;
        match self.cmd.unwrap_or_else(||{
            info!("Use `{BIN_NAME}` without subcommand defaults to `{BIN_NAME} paste`. If this is not intended, add a subcommand. Use `{BIN_NAME} -h` for more information.");
            Default::default()
        }) {
            Cmd::Auth { auth } => {
                let config = Config::new(auth);
                info!("Verifying new auth info...");
                config.as_api()?.verify_token()?;

                if flag.dry_run {
                    info!("with --dry-run, furthur actions are avoided.");
                    return Ok(())
                }

                config.write_to(config_path)?;
                info!("Done adding authentication!");
                Ok(())
            }
            Cmd::Paste { file_name, metadata } => {
                let config = Config::from_dir(config_path)?;
                let api = config.as_api()?;
                let mut cb = Clipboard::new()?;
                let filename = file_name.unwrap_or_else(image_name);

                let image = match cb.get_image() {
                    Ok(image) => image,
                    Err(e) => {
                        error!("Failed to retrieve image data from clipboard");
                        error!("{}", e);
                        return Ok(());
                    }
                };

                let (w, h) = (image.width, image.height);
                let png = image_data_to_png(image)?;
                let size = bytesize::to_string(png.len().try_into()?, true);

                info!("Image in clipboard: {} x {}, {}", w.green(), h.green(), size.blue());

                let mut upload = api.upload(&filename, &png);
                upload.extend_meta(metadata.iter().map(|x| (x.k.as_str(), x.v.as_str())));

                if flag.dry_run {
                    info!("with --dry-run, furthur actions are avoided.");
                    return Ok(())
                }

                info!("Uploading image...");

                upload.send().wrap_err("Failed to upload image")?.log();

                Ok(())
            }

            Cmd::Upload {
                file_path,
                metadata,
                file_name
            } => {
                let config = Config::from_dir(config_path)?;
                let api = config.as_api()?;

                let img = Reader::open(&file_path).wrap_err("Failed to open img file")?.decode().wrap_err("Unsupported img format")?;
                let mut buf = Vec::with_capacity(img.as_bytes().len());
                img.write_to(&mut buf, image::ImageFormat::Png).wrap_err("Unable to encode image")?;

                let (w, h) = img.dimensions();
                let size = bytesize::to_string(buf.len().try_into()?, true);

                let filename = file_name.or_else(||
                    file_path.file_stem().and_then(|x| x.to_str().map(ToOwned::to_owned).map(|x| x + ".png"))
                ).unwrap_or_else(image_name);

                info!("Image ({}): {} x {}, {}", filename, w.green(), h.green(), size.blue());

                let mut upload = api.upload(&filename, &buf);
                upload.extend_meta(metadata.iter().map(|x| (x.k.as_str(), x.v.as_str())));

                if flag.dry_run {
                    info!("with --dry-run, furthur actions are avoided.");
                    return Ok(())
                }

                info!("Uploading image...");

                upload.send().wrap_err("Failed to upload image")?.log();

                Ok(())
            },
        }
    }
}
