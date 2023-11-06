use pollutionclient_rs::*;
use std::{thread, time::Duration};
use influxdb::{Client, Error};
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let running_config: Config = match Config::parse_env() {
        Ok(configuration) => configuration,
        Err(e) => panic!("Unable to set configuration. Error returned: {e}"),
    };
    if running_config.get_key() == "NOAPISET".to_string() {
        panic!("API key is not set. Unable to proceed.")
    };
    if running_config.location_is_set() {
        println!("Location added: {}", running_config.get_location())
    } else {
        panic!("Location not set. Unable to proceed.")
    };

    let running_coords: [String; 2] = running_config.get_coords();
    match running_coords[0].parse::<f32>() {
        Ok(_) => println!("Latitude looks good."),
        Err(e) => panic!("Latitude looks malformed. {} given but parsing returns: {}", running_coords[0], e),
    }
    match running_coords[1].parse::<f32>() {
        Ok(_) => println!("Longitude looks good."),
        Err(e) => panic!("Longitude looks malformed. {} given but parsing returns: {}", running_coords[1], e),
    }

    println!("InfluxDB server set to: {}", running_config.get_dbserver());
    println!("If this is incorrect, ensure that OPENWEATHER_INFLUXDB_SERVER is set correctly.");
    println!("InfluxDB name set to {}", running_config.get_dbname());
    println!("If this is incorrect, ensure that OPENWEATHER_INFLUXDB_NAME is set correctly.");

    let running_client: Client = build_client(&running_config);

    let running_url: String = format!("http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}", &running_coords[0], &running_coords[1], running_config.get_key());

    let mut error_count: u8 = 0;

    while error_count < running_config.get_maxretry() {
        let response: Result<PollResponse, ureq::Error> = match get_pollution(&running_url) {
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        };

        if response.is_ok() {
            let unpacked: PollResponse = response.unwrap();
            let results: PollUpdate = unpacked.unpack();

            write_to_db(&running_client, results).await?;

            println!("Successfully written to DB {}", running_config.get_dbname());
            thread::sleep(Duration::from_secs(running_config.get_timing()));
        } else {
            println!("Error encountered while grabbing stats.");
            error_count = error_count + 1;
            match response.unwrap_err() {
                ureq::Error::Status(code, resp) => println!("Status: {}, Text: {}", code, resp.status_text()),
                ureq::Error::Transport(trans) => println!("Kind: {}, Message: {}", trans.kind(), trans.message().unwrap_or("N/A")),
            };
            thread::sleep(Duration::from_secs(running_config.get_timing() / 2));
        } 
    }
    panic!("Max errors reached! Terminating loop and script.");
}