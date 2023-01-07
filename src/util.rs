use std::time;

use arboard::ImageData;
use color_eyre::{
    config::HookBuilder,
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
    Result,
};
use humantime::{format_rfc3339, format_rfc3339_seconds};
use image::{codecs::png::PngEncoder, ColorType, EncodableLayout, ImageEncoder, RgbaImage};
use log::{error, info};
use url::Url;

use crate::{Image, Response};

pub fn image_data_to_png(data: ImageData) -> Result<Vec<u8>> {
    let size = data.bytes.len();
    let width: u32 = data.width.try_into().wrap_err("Image width too big")?;
    let height: u32 = data.height.try_into().wrap_err("Image height too big")?;
    let img = RgbaImage::from_raw(width, height, data.bytes.to_vec())
        .wrap_err("Bad `ImageData`")
        .wrap_err("Unable to convert raw pixels to encodable RgbaImage")?;
    let mut buf = Vec::with_capacity(size);
    PngEncoder::new(&mut buf).write_image(img.as_bytes(), width, height, ColorType::Rgba8)?;
    Ok(buf)
}

pub fn image_name() -> String {
    let now = time::SystemTime::now();
    format_rfc3339_seconds(now).to_string() + ".png"
}

pub fn display_title(title: &str, space: usize) {
    println!();
    display_aligned("", &title.bold().to_string(), space)
}

pub fn display_aligned(k: &str, v: &str, space: usize) {
    println!(" {:>space$}  {}", k.blue().bold(), v)
}

pub fn format_markdown_url(url: &Url, filename: &str) -> String {
    format!("![{}]({})", filename, url.as_str())
}
pub fn format_html_url(url: &Url, filename: &str) -> String {
    format!("<img alt=\"{}\" src=\"{}\" />", filename, url.as_str())
}

mod logger {
    use color_eyre::{eyre::Context, Result};
    use env_logger::{
        filter::Builder,
        fmt::{Color, Style, StyledValue},
    };
    use log::Level;

    pub fn init_logger() -> Result<()> {
        let mut builder = env_logger::Builder::new();

        builder
            .format(|f, record| {
                use std::io::Write;

                let mut style = f.style();
                let level = colored_level(&mut style, record.level());

                writeln!(f, " {}  {}", level, record.args())
            })
            .filter_level({
                match ::std::env::var("SHOT_LOG") {
                    Ok(filter) => Builder::default().parse(&filter).build().filter(),
                    Err(_) => log::LevelFilter::Info,
                }
            })
            .try_init()
            .wrap_err("Failed to init logger")
    }

    fn colored_level(style: &mut Style, level: Level) -> StyledValue<&'static str> {
        match level {
            Level::Trace => style
                .set_bold(true)
                .set_color(Color::Magenta)
                .value("TRACE"),
            Level::Debug => style.set_bold(true).set_color(Color::Blue).value("DEBUG"),
            Level::Info => style.set_bold(true).set_color(Color::Green).value(" INFO"),
            Level::Warn => style.set_bold(true).set_color(Color::Yellow).value(" WARN"),
            Level::Error => style.set_bold(true).set_color(Color::Red).value("ERROR"),
        }
    }
}

pub use logger::init_logger;

impl Response<Image> {
    pub fn log(&self) {
        if let Some(ref msgs) = self.messages {
            msgs.iter()
                .for_each(|msg| info!("{}  {}", "Message".blue(), msg))
        }

        if !self.success {
            error!("API returned an error:");

            self.errors
                .iter()
                .for_each(|err| error!("(Code {}) {}", err.code.red(), err.message));
        } else if let Some(ref img) = self.result {
            let space = 5;
            info!("Image uploaded.");
            display_title("General", space);
            display_aligned("ID", &img.id, space);
            display_aligned("Name", &img.filename, space);
            display_aligned("Time", &format_rfc3339(img.uploaded).to_string(), space);

            if let Some(ref md) = img.meta {
                if !md.is_empty() {
                    display_title("Metadata", space);
                    md.iter().for_each(|(k, v)| display_aligned(k, v, space))
                }
            }

            img.variants.iter().for_each(|url| {
                let variant = url
                    .path_segments()
                    .and_then(|x| x.last())
                    .unwrap_or("UNKNOWN");

                display_title(&format!("Variant {}", variant.green()), space);
                display_aligned("Url", url.as_str(), space);
                display_aligned("HTML", &format_html_url(url, &img.filename), space);
                display_aligned("MD", &format_markdown_url(url, &img.filename), space);
            })
        } else {
            error!("Bad response: {:#?}", self)
        };
    }
}

pub fn init() -> Result<()> {
    HookBuilder::default()
        .display_env_section(false)
        .install()?;
    init_logger()?;
    Ok(())
}
