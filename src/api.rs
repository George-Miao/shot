use std::collections::HashMap;

use attohttpc::{MultipartBuilder, MultipartFile};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use log::debug;
use url::Url;

use crate::{Auth, Image, Response};

#[derive(Debug)]
pub struct API {
    account_id: String,
    token: String,
    session: attohttpc::Session,
    api: Url,
}

impl API {
    const API: &'static str = "https://api.cloudflare.com/client/v4/";

    pub fn new(auth: impl Into<Auth>) -> Result<Self> {
        let Auth { account_id, token } = auth.into();
        let this = Self {
            api: Url::parse(Self::API)
                .unwrap()
                .join(&format!("accounts/{}/images/v1", account_id))
                .wrap_err("Bad `account_id`")?,
            account_id,
            token,
            session: Default::default(),
        };
        Ok(this)
    }

    pub fn verify_token(&self) -> Result<()> {
        let url = Url::parse(Self::API)
            .unwrap()
            .join(&format!("accounts/{}/images/v1", self.account_id()))
            .wrap_err("Bad `account_id`")?;
        let res = self
            .session
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .wrap_err("Failed to request API")?;
        let is_success = res.is_success();
        let res_text = res.text().wrap_err("Bad response")?;
        debug!("{}", res_text);
        if is_success {
            Ok(())
        } else {
            Err(eyre!("{}", res_text).wrap_err("Unable to verify the auth pair"))
        }
    }

    /// Get the Cloudflare Image API url
    pub fn url(&self) -> &Url {
        &self.api
    }

    /// Get the Cloudflare Account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn get_direct_upload(&self) -> Result<()> {
        todo!()
    }

    pub fn upload<'a>(&'a self, filename: &'a str, bytes: &'a [u8]) -> Upload<'a> {
        Upload::new(self, filename, bytes)
    }
}

/// Upload builder. Use [`send`] to perform the upload.
pub struct Upload<'a> {
    api: &'a API,
    filename: &'a str,
    bytes: &'a [u8],
    require_signed_urls: bool,
    meta: HashMap<&'a str, &'a str>,
}

impl<'a> Upload<'a> {
    fn new(api: &'a API, filename: &'a str, bytes: &'a [u8]) -> Self {
        Self {
            api,
            filename,
            bytes,
            require_signed_urls: Default::default(),
            meta: Default::default(),
        }
    }

    pub fn bytes(self, bytes: &'a [u8]) -> Self {
        Self { bytes, ..self }
    }

    pub fn filename(self, filename: &'a str) -> Self {
        Self { filename, ..self }
    }

    pub fn require_signed_urls(self) -> Self {
        Self {
            require_signed_urls: true,
            ..self
        }
    }

    pub fn add_meta(&mut self, key: &'a str, value: &'a str) -> &mut Self {
        self.meta.insert(key, value);
        self
    }

    pub fn extend_meta(&mut self, meta: impl Iterator<Item = (&'a str, &'a str)>) -> &mut Self {
        self.meta.extend(meta);
        self
    }

    pub fn send(self) -> Result<Response<Image>> {
        let url = self.api.url();

        debug!("API Url: {}", url);

        let file = MultipartFile::new("file", self.bytes)
            .with_filename(self.filename)
            .with_type("image/png")?;

        let signed = self.require_signed_urls.to_string();
        let meta = serde_json::to_string(&self.meta).wrap_err("Failed to serialize metadata")?;

        let form = MultipartBuilder::new()
            .with_file(file)
            .with_text("requireSignedURLs", &signed)
            .with_text("metadata", &meta)
            .build()?;

        let res = self
            .api
            .session
            .post(url)
            .body(form)
            .bearer_auth(&self.api.token)
            .send()
            .wrap_err("Failed to request API")?
            .json()
            .wrap_err("Failed to parse response json")?;

        debug!("Res: {:#?}", res);

        Ok(res)
    }
}
