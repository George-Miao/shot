use std::{collections::HashMap, time::SystemTime};

use serde::{Deserialize as De, Serialize as Se};
use url::Url;

#[derive(Se, De, Debug, Clone)]
pub struct Response<T> {
    pub success: bool,
    pub result: Option<T>,
    pub result_info: Option<String>,
    pub messages: Option<Vec<String>>,
    pub errors: Vec<ApiError>,
}

#[derive(Se, De, Debug, Clone)]
pub struct Image {
    pub id: String,
    pub filename: String,
    #[serde(rename = "requireSignedURLs")]
    pub require_signed_urls: bool,
    #[serde(with = "humantime_serde")]
    pub uploaded: SystemTime,
    pub variants: Vec<Url>,
    pub meta: Option<HashMap<String, String>>,
}

#[derive(Se, De, Debug, Clone)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}
