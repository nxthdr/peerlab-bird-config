use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub headscale_api_url: String,
    pub headscale_api_key: String,
    pub output_file: PathBuf,
    pub reload_bird: bool,
}
