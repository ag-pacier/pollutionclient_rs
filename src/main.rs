use ureq;
use std::{env, thread, time::Duration};
use std::fmt;
use serde::Deserialize;

#[derive(Clone, Debug)]
struct Config {
    apikey: Option<String>,
    location: Option<ZipLoc>,
    timing: u64,
}

impl Config {
    fn new() -> Config {
        Config { apikey: None, location: None, timing: 60 }
    }
    fn set_loc(&mut self, new_loc: ZipLoc) -> () {
        self.location = Some(new_loc);
    }
    fn set_key(&mut self, new_key: String) -> () {
        self.apikey = Some(new_key);
    }
    fn set_timing(&mut self, new_timing: u64) -> () {
        self.timing = new_timing;
    }
    fn get_key(&self) -> String {
        match &self.apikey {
            Some(key) => key.to_owned(),
            None => "NOAPISET".to_string(),
        }
    }
    fn get_coords(&self) -> [String; 2] {
        match &self.location {
            Some(loc) => [loc.lat.to_string(), loc.lon.to_string()],
            None => ["0".to_string(), "0".to_string()],
        }
    }
    fn get_timing(&self) -> u64 {
        self.timing
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
            current_config.set_loc(env_location);
        };
        let config_timing: String = match env::var("OPENWEATHER_POLL_TIMING") {
            Ok(timing) => timing,
            Err(_) => "3600".to_string(),
        };
        current_config.set_timing(config_timing.parse::<u64>().unwrap_or(3600));
        Ok(current_config)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ZipLoc {
    zip: String,
    name: String,
    lat: f32,
    lon: f32,
    country: String,
}

impl fmt::Display for ZipLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Zip-Code: {}, Country: {}, City: {}, Lat: {}, Lon: {}", self.zip, self.country, self.name, self.lat, self.lon)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Components {
    co: f32,
    no: f32,
    no2: f32,
    o3: f32,
    so2: f32,
    pm2_5: f32,
    pm10: f32,
    nh3: f32,
}
impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Carbon Monoxide: {} μg/m3, Nitrogen Monoxide: {} μg/m3, Nitrogen Dioxide: {} μg/m3, Ozone: {} μg/m3, Sulphur Dioxide: {} μg/m3, Fine Particulate Matter: {} μg/m3, Course Particulate Matter: {} μg/m3, Ammonia: {} μg/m3",
        self.co, self.no, self.no2, self.o3, self.so2, self.pm2_5, self.pm10, self.nh3)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct MainAqi {
    aqi: i8,
}
impl fmt::Display for MainAqi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Air Quality: {}", self.aqi)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct PollList {
    components: Components,
    main: MainAqi,
}

impl fmt::Display for PollList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AQI: {}, Components: {}", self.main.aqi, self.components)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct PollResponse {
    list: Vec<PollList>,
}

impl fmt::Display for PollResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "List: {:#?}", self.list)
    }
}

fn get_coords_zipcode(zip: String, country: String, apikey: String) -> Result<ZipLoc, ureq::Error> {
    let url: String = format!("http://api.openweathermap.org/geo/1.0/zip?zip={zip},{country}&appid={apikey}");
    let response: ZipLoc = ureq::get(&url).call()?.into_json()?;
    Ok(response)
}

fn get_pollution(url: &str) -> Result<PollResponse, ureq::Error> {
    let response: PollResponse = ureq::get(url).call()?.into_json()?;
    Ok(response)
}

fn main() {
    let running_config: Config = match Config::parse_env() {
        Ok(configuration) => configuration,
        Err(e) => panic!("Unable to set configuration. Error returned: {e}"),
    };
    if running_config.apikey.is_none() {
        panic!("API key is not set using environmental variable. Unable to proceed. Please set OPENWEATHER_API_KEY and try again.")
    };
    match &running_config.location {
        Some(conf_loc) => println!("Location added: {}", conf_loc),
        None => panic!("Location not set using environmental variables. Unable to proceed. Please set OPENWEATHER_POLL_ZIP and if not in the US, OPENWEATHER_POLL_COUNTRY and try again!")
    };

    let running_coords: [String; 2] = running_config.get_coords();
    if running_coords == ["0".to_string(), "0".to_string()] {
        panic!("Default coordinates obtained. Likely unable to find correct location. Please double check location vars and try again.")
    };

    let running_url: String = format!("http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}", &running_coords[0], &running_coords[1], running_config.get_key());
    loop {
        let response: PollResponse = match get_pollution(&running_url) {
            Ok(res) => res,
            Err(ureq::Error::Status(code, res)) => panic!("Server returned: {} with a response: {:?}", code, res),
            Err(e) => panic!("Internal error: {}", e),
        };
        let current_aqi: MainAqi = response.list[0].main.clone();
        let current_pollution: Components = response.list[0].components.clone();
        println!("Current AQI: {:#?}", current_aqi);
        println!("Component breakdown: {}", current_pollution);
        thread::sleep(Duration::from_secs(running_config.get_timing()));
    }
}
