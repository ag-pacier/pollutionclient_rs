use ureq;
use std::env;
use serde::Deserialize;

#[derive(Clone, Debug)]
struct Config {
    apikey: Option<String>,
    location: Option<ZipLoc>,
}

impl Config {
    fn new() -> Config {
        Config { apikey: None, location: None }
    }
    fn set_loc(&mut self, new_loc: ZipLoc) -> () {
        self.location = Some(new_loc);
    }
    fn set_key(&mut self, new_key: String) -> () {
        self.apikey = Some(new_key);
    }
    fn get_key(&self) -> String {
        match &self.apikey {
            Some(key) => key.to_owned(),
            None => "None".to_string(),
        }
    }
    fn get_coords(&self) -> [String; 2] {
        let current_location: ZipLoc = match &self.location {
            Some(loc) => loc.clone(),
            None => ZipLoc { zip: "0".to_string(), name: "0".to_string(), lat: "0".to_string(), lon: "0".to_string(), country: "0".to_string() },
        };
        [current_location.lat, current_location.lon]
    }
    fn parse_env() -> Result<Config, ureq::Error> {
        let mut current_config: Config = Config::new();
        let new_api_key: Option<String> = match env::var("OPENWEATHER_API_KEY") {
            Ok(key) => Some(key),
            Err(_) => None,
        };
        if new_api_key.is_some() {
            current_config.set_key(new_api_key.unwrap());
        };
        let zip_code: Option<String> = match env::var("OPENWEATHER_POLL_ZIP") {
            Ok(set_zip) => Some(set_zip),
            Err(_) => None,
        };
        if zip_code.is_some() {
            let country: String = match env::var("OPENWEATHER_POLL_COUNTRY") {
                Ok(set_country) => set_country,
                Err(_) => "US".to_string(),
            };
            let env_location = get_coords_zipcode(zip_code.unwrap(), country, current_config.get_key())?;
            current_config.location = Some(env_location);
        }
        Ok(current_config)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ZipLoc {
    zip: String,
    name: String,
    lat: String,
    lon: String,
    country: String,
}

fn get_coords_zipcode(zip: String, country: String, apikey: String) -> Result<ZipLoc, ureq::Error> {
    let url: String = format!("http://api.openweathermap.org/geo/1.0/zip?zip={zip},{country}&appid={apikey}");
    let response: ZipLoc = ureq::get(&url).call()?.into_json()?;
    Ok(response)
}

fn main() {
    let running_config: Config = Config::parse_env().unwrap();
    
}
