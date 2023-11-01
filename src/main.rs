use ureq;
use std::{env, thread, time::Duration};
use std::fmt;
use serde::Deserialize;
use influxdb::{Client, WriteQuery, Error};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};
use pollster::FutureExt as _;

#[derive(Clone, Debug)]
struct Config {
    apikey: Option<String>,
    location: Option<ZipLoc>,
    timing: u64,
    dbname: Option<String>,
    dbserver: Option<String>,
    dbuser: Option<String>,
    dbpass: Option<String>,
}

impl Config {
    fn new() -> Config {
        Config { apikey: None, location: None, timing: 60, dbname: None, dbserver: None, dbuser: None, dbpass: None }
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
    fn set_dbname(&mut self, new_dbname: String) -> () {
        self.dbname = Some(new_dbname);
    }
    fn set_dbuser(&mut self, new_dbuser: String) -> () {
        self.dbuser = Some(new_dbuser);
    }
    fn set_dbpass(&mut self, new_dbpass: String) -> () {
        self.dbpass = Some(new_dbpass);
    }
    fn set_dbserver(&mut self, new_dbserver: String) -> () {
        let mut final_server: String = format!("{}", &new_dbserver);
        if !final_server.starts_with("http://") {
            final_server = format!("http://{}", final_server);
        };
        let colon_check: Vec<&str> = new_dbserver.rsplit(":").collect();
        if colon_check.len() < 3 {
            final_server = format!("{}:8086", final_server);
        }
        self.dbserver = Some(final_server);
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
    fn get_dbserver(&self) -> String {
        match &self.dbserver {
            Some(server) => server.to_owned(),
            None => "http://localhost:8086".to_string(),
        }
    }
    fn get_dbname(&self) -> String {
        match &self.dbname {
            Some(name) => name.to_owned(),
            None => "test".to_string(),
        }
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
        let new_dbname: Option<String> = match env::var("OPENWEATHER_INFLUXDB_NAME") {
            Ok(name) => Some(name),
            Err(_) => None,
        };
        if new_dbname.is_some() {
            current_config.set_dbname(new_dbname.unwrap());
        };
        let new_dbserver: Option<String> = match env::var("OPENWEATHER_INFLUXDB_SERVER") {
            Ok(name) => Some(name),
            Err(_) => None,
        };
        if new_dbserver.is_some() {
            current_config.set_dbserver(new_dbserver.unwrap());
        };
        let new_dbuser: Option<String> = match env::var("OPENWEATHER_INFLUXDB_DBUSER") {
            Ok(name) => Some(name),
            Err(_) => None,
        };
        if new_dbuser.is_some() {
            current_config.set_dbuser(new_dbuser.unwrap());
        };
        let new_dbpass: Option<String> = match env::var("OPENWEATHER_INFLUXDB_DBPASS") {
            Ok(pass) => Some(pass),
            Err(_) => None,
        };
        if new_dbpass.is_some() {
            current_config.set_dbpass(new_dbpass.unwrap());
        };
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

#[derive(InfluxDbWriteable)]
struct PollUpdate {
    time: DateTime<Utc>,
    #[influxdb(tag)] aqi: i8,
    co: f32,
    no: f32,
    no2: f32,
    o3: f32,
    so2: f32,
    pm2_5: f32,
    pm10: f32,
    nh3: f32,
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

async fn write_to_db(dbclient: &Client, aqi: i8, pollution: Components) -> Result<String, Error> {
    let dbupdate: WriteQuery = PollUpdate { time: Utc::now(),
         aqi: aqi, co: pollution.co, no: pollution.no, no2: pollution.no2, o3: pollution.o3, so2: pollution.so2,
         pm2_5: pollution.pm2_5, pm10: pollution.pm10, nh3: pollution.nh3 }.into_query("pollution");

    let internal_client: Client = dbclient.clone();
    
    let result: String = internal_client.query(dbupdate).await?;

    Ok(result)
}

fn build_client(current_config: &Config) -> Client {
    let this_config: Config = current_config.clone();
    if this_config.dbpass.is_none() {
        match &this_config.dbuser {
            Some(_) => panic!("InfluxDB user set but password is not. PLease add OPENWEATHER_INFLUXDB_DBPASS and try again"),
            None => println!("InfluxDB authentication not added. If this is a mistake, set OPENWEATHER_INFLUXDB_DBUSER and OPENWEATHER_INFLUXDB_DBPASS and try again")
        };
    } else {
        match &this_config.dbuser {
            Some(conf_user) => println!("InfluxDB user added: {}", conf_user),
            None => panic!("InfluxDB user not added but password added. Set OPENWEATHER_INFLUXDB_DBUSER and OPENWEATHER_INFLUXDB_DBPASS and try again")
        };
    }

    if this_config.dbpass.is_some() {
        Client::new(this_config.get_dbserver(), this_config.get_dbname()).with_auth(&this_config.dbuser.clone().unwrap(), &this_config.dbpass.clone().unwrap())
    } else {
        Client::new(this_config.get_dbserver(), this_config.get_dbname())
    }
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

    match &running_config.dbserver {
        Some(conf_server) => println!("InfluxDB added: {}", conf_server),
        None => panic!("DBServer not set using environmental variables. Unable to proceed. Please set OPENWEATHER_INFLUXDB_SERVER and try again!")
    };
    match &running_config.dbname {
        Some(conf_name) => println!("InfluxDB name added: {}", conf_name),
        None => panic!("DBServer not set using environmental variables. Unable to proceed. Please set OPENWEATHER_INFLUXDB_NAME and try again!")
    };

    let running_client: Client = build_client(&running_config);

    let running_url: String = format!("http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}", &running_coords[0], &running_coords[1], running_config.get_key());
    loop {
        let response: PollResponse = match get_pollution(&running_url) {
            Ok(res) => res,
            Err(ureq::Error::Status(code, res)) => panic!("Server returned: {} with a response: {:?}", code, res),
            Err(e) => panic!("Internal error: {}", e),
        };
        let current_aqi: MainAqi = response.list[0].main.clone();
        let current_pollution: Components = response.list[0].components.clone();
        println!("{}", current_aqi);
        println!("Component breakdown: {}", current_pollution);

        let my_fut = async {write_to_db(&running_client, current_aqi.aqi, current_pollution).await};
        let result = my_fut.block_on();
    
        if result.is_err() {
            panic!("Failed to write to DB! Error: {}", result.unwrap_err());
        } else {
            println!("DB Write passed: {}", result.unwrap());
        }

        thread::sleep(Duration::from_secs(running_config.get_timing()));
    }
}
