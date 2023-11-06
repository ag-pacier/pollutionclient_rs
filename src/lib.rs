use ureq;
use std::env;
use std::fmt;
use serde::Deserialize;
use influxdb::{Client, WriteQuery, Error};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};


#[derive(Clone, Debug)]
pub struct Config {
    apikey: Option<String>,
    location: Option<ZipLoc>,
    timing: u64,
    dbname: Option<String>,
    dbserver: Option<String>,
    dbuser: Option<String>,
    dbpass: Option<String>,
    max_retry: u8,
}

impl Config {
    fn new() -> Config {
        Config { apikey: None, location: None, timing: 3600, dbname: None, dbserver: None, dbuser: None, dbpass: None, max_retry: 3 }
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
        let colon_check: Vec<&str> = final_server.rsplit(":").collect();
        if colon_check.len() < 3 {
            final_server = format!("{}:8086", final_server);
        }
        self.dbserver = Some(final_server);
    }
    fn set_maxretry(&mut self, new_retry: u8) -> () {
        self.max_retry = new_retry;
    }
    pub fn get_key(&self) -> String {
        match &self.apikey {
            Some(key) => key.to_owned(),
            None => "NOAPISET".to_string(),
        }
    }
    pub fn get_coords(&self) -> [String; 2] {
        match &self.location {
            Some(loc) => [loc.lat.to_string(), loc.lon.to_string()],
            None => ["NOTSET".to_string(), "NOTSET".to_string()],
        }
    }
    pub fn get_location(&self) -> String {
        self.location.clone().unwrap().to_string()
    }
    pub fn get_timing(&self) -> u64 {
        self.timing
    }
    pub fn get_dbserver(&self) -> String {
        match &self.dbserver {
            Some(server) => server.to_owned(),
            None => "http://localhost:8086".to_string(),
        }
    }
    pub fn get_dbname(&self) -> String {
        match &self.dbname {
            Some(name) => name.to_owned(),
            None => "test".to_string(),
        }
    }
    pub fn get_maxretry(&self) -> u8 {
        self.max_retry.clone()
    }
    pub fn location_is_set(&self) -> bool {
        match self.location {
            Some(_) => true,
            None => false,
        }
    }
    pub fn parse_env() -> Result<Config, ureq::Error> {
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
        let new_maxretry: String = match env::var("OPENWEATHER_MAX_RETRY") {
            Ok(max_retry) => max_retry,
            Err(_) => "3".to_string(),
        };
        current_config.set_maxretry(new_maxretry.parse::<u8>().unwrap_or(3));
        Ok(current_config)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
pub struct Components {
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
pub struct MainAqi {
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
pub struct PollResponse {
    list: Vec<PollList>,
}

impl fmt::Display for PollResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "List: {:#?}", self.list)
    }
}

impl PollResponse {
    pub fn unpack(self) -> PollUpdate {
        let current_aqi: MainAqi = self.list[0].main.clone();
        let current_pollution: Components = self.list[0].components.clone();
        println!("{}", current_aqi);
        println!("Component breakdown:");
        println!("{}", current_pollution);
        PollUpdate { time: Utc::now(),
            aqi: current_aqi.aqi, co: current_pollution.co, no: current_pollution.no, no2: current_pollution.no2, o3: current_pollution.o3, so2: current_pollution.so2,
            pm2_5: current_pollution.pm2_5, pm10: current_pollution.pm10, nh3: current_pollution.nh3 }

    }
}

#[derive(InfluxDbWriteable)]
pub struct PollUpdate {
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

pub fn get_pollution(url: &str) -> Result<PollResponse, ureq::Error> {
    let response: PollResponse = ureq::get(url).call()?.into_json()?;
    Ok(response)
}

pub async fn write_to_db(dbclient: &Client, pollution: PollUpdate) -> Result<String, Error> {
    let dbupdate: WriteQuery = pollution.into_query("pollution");

    let internal_client: Client = dbclient.clone();
    
    let result: String = internal_client.query(dbupdate).await?;

    Ok(result)
}

pub fn build_client(current_config: &Config) -> Client {
    let this_config: Config = current_config.clone();
    if this_config.dbpass.is_none() {
        match &this_config.dbuser {
            Some(_) => panic!("InfluxDB user set but password is not."),
            None => println!("InfluxDB authentication not added due to blank USER/PASS configuration.")
        };
    } else {
        match &this_config.dbuser {
            Some(conf_user) => println!("InfluxDB user added: {}", conf_user),
            None => panic!("InfluxDB password added but not user! Unable to proceed.")
        };
    }

    if this_config.dbpass.is_some() {
        Client::new(this_config.get_dbserver(), this_config.get_dbname()).with_auth(&this_config.dbuser.clone().unwrap(), &this_config.dbpass.clone().unwrap())
    } else {
        Client::new(this_config.get_dbserver(), this_config.get_dbname())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_config_defaults() {
        let test_config: Config = Config::new();
        assert_eq!(test_config.timing, 3600);
        assert_eq!(test_config.max_retry, 3);
    }

    #[test]
    fn config_set_loc_works() {
        let mut test_config: Config = Config::new();
        let new_zipcode: ZipLoc = ZipLoc { zip: "00000".to_string(), name: "test".to_string(), lat: 42.0, lon: 42.0, country: "US".to_string() };
        test_config.set_loc(new_zipcode.clone());
        assert_eq!(test_config.location.unwrap(), new_zipcode);
    }

    #[test]
    fn config_set_key_works() {
        let mut test_config: Config = Config::new();
        let test_key: String = "BigTestString".to_string();
        test_config.set_key(test_key.clone());
        assert_eq!(test_config.apikey.unwrap(), test_key);
    }

    #[test]
    fn config_set_timing_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_timing: u64 = 32;
        test_config.set_timing(new_timing.clone());
        assert_eq!(test_config.timing, new_timing);
        assert_ne!(test_config.timing, control_config.timing);
    }

    #[test]
    fn config_get_timing_default() {
        let test_config: Config = Config::new();
        let current_default: u64 = 3600;
        assert_eq!(test_config.get_timing(), current_default);
    }

    #[test]
    fn config_set_dbname_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbname: String = "testThisdata".to_string();
        test_config.set_dbname(new_dbname.clone());
        assert_eq!(test_config.dbname, Some(new_dbname));
        assert_ne!(test_config.dbname, control_config.dbname);
    }

    #[test]
    fn config_set_dbserver_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbserver: String = "http://testThisdata:8080".to_string();
        test_config.set_dbserver(new_dbserver.clone());
        assert_eq!(test_config.dbserver, Some(new_dbserver));
        assert_ne!(test_config.dbserver, control_config.dbserver);
    }

    #[test]
    fn config_set_dbserver_works_no_http() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbserver: String = "testThisdata:8080".to_string();
        test_config.set_dbserver(new_dbserver.clone());
        assert_eq!(test_config.dbserver, Some(format!("http://{}", new_dbserver)));
        assert_ne!(test_config.dbserver, Some(new_dbserver));
        assert_ne!(test_config.dbserver, control_config.dbserver);
    }

    #[test]
    fn config_set_dbserver_works_no_port() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbserver: String = "http://testThisdata".to_string();
        test_config.set_dbserver(new_dbserver.clone());
        assert_eq!(test_config.dbserver, Some(format!("{}:8086", new_dbserver)));
        assert_ne!(test_config.dbserver, Some(new_dbserver));
        assert_ne!(test_config.dbserver, control_config.dbserver);
    }

        #[test]
    fn config_set_dbserver_works_no_port_or_http() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbserver: String = "testThisdata".to_string();
        test_config.set_dbserver(new_dbserver.clone());
        assert_eq!(test_config.dbserver, Some(format!("http://{}:8086", new_dbserver)));
        assert_ne!(test_config.dbserver, Some(new_dbserver));
        assert_ne!(test_config.dbserver, control_config.dbserver);
    }

    #[test]
    fn config_set_dbuser_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbuser: String = "testThisdata".to_string();
        test_config.set_dbuser(new_dbuser.clone());
        assert_eq!(test_config.dbuser, Some(new_dbuser));
        assert_ne!(test_config.dbuser, control_config.dbuser);
    }

    #[test]
    fn config_set_dbpass_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_dbpass: String = "testThisdata".to_string();
        test_config.set_dbpass(new_dbpass.clone());
        assert_eq!(test_config.dbpass, Some(new_dbpass));
        assert_ne!(test_config.dbpass, control_config.dbpass);
    }

    #[test]
    fn config_set_maxretry_works() {
        let mut test_config: Config = Config::new();
        let control_config: Config = Config::new();
        let new_max_retry: u8 = 55;
        test_config.set_maxretry(new_max_retry);
        assert_eq!(test_config.max_retry, new_max_retry);
        assert_ne!(test_config.max_retry, control_config.max_retry);
    }

    #[test]
    fn config_api_none_by_new() {
        let test_config: Config = Config::new();
        assert_eq!(test_config.apikey, None);
    }

    #[test]
    fn config_api_none_get_key_noapiset() {
        let test_config: Config = Config::new();
        assert_eq!(test_config.apikey, None);
        assert_eq!(test_config.get_key(), "NOAPISET".to_string());
    }

    #[test]
    fn config_api_some_get_key() {
        let mut test_config: Config = Config::new();
        let new_key: String = "NewTestKey".to_string();
        test_config.set_key(new_key.clone());
        assert_eq!(test_config.apikey, Some(new_key));
    }

    #[test]
    fn config_get_coords_none() {
        let test_config: Config = Config::new();
        let test_coords: [String; 2] = test_config.get_coords();
        assert_eq!(test_coords, ["NOTSET".to_string(), "NOTSET".to_string()])
    }

    #[test]
    #[should_panic]
    fn config_get_coords_none_parsing() {
        let test_config: Config = Config::new();
        let test_coords: [String; 2] = test_config.get_coords();
        let _parse_check: f32 = test_coords[0].parse().unwrap();
    }

    #[test]
    fn config_get_coords_some() {
        let control_config: Config = Config::new();
        let control_coords: [String; 2] = control_config.get_coords();
        let accurate_coords: [f32; 2] = [42.5, 42.5];
        let test_zip: ZipLoc = ZipLoc { zip: "99999".to_string(), name: "TestLoc".to_string(), lat: accurate_coords[0], lon: accurate_coords[1], country: "US".to_string() };
        let test_config: Config = Config { apikey: None, location: Some(test_zip), timing: 5, dbname: None, dbserver: None, dbuser: None, dbpass: None, max_retry: 3 };
        let test_coords: [String; 2] = test_config.get_coords();
        let parsed_test_coords: [f32; 2] = [test_coords[0].parse().unwrap(), test_coords[1].parse().unwrap()];
        assert_eq!(accurate_coords, parsed_test_coords);
        assert_ne!(control_coords, test_coords);
    }

    #[test]
    fn config_get_dbserver_default() {
        let test_config: Config = Config::new();
        let dbdefault: String = test_config.get_dbserver();
        assert_eq!(dbdefault, "http://localhost:8086".to_string());
    }

    #[test]
    fn config_get_dbname_default() {
        let test_config: Config = Config::new();
        let dbdefault: String = test_config.get_dbname();
        assert_eq!(dbdefault, "test".to_string());
    }

}