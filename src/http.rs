use reqwest::blocking::Client;

pub type Error = reqwest::Error;
pub type Response = reqwest::blocking::Response;

pub fn get(url: &str) -> Result<Response, Error> {
    Client::new()
        .get(url)
        .header("User-Agent", concat!("cmvm ", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
}