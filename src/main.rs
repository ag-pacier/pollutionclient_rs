use pollutionclient_rs::*;
use std::{thread, time::Duration, env};
use influxdb::{Client, Error};
use tokio;

// Utilizing tokio as "current_thread" to ensure async function is taken care of. It's okay that it's actually blocking.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // Check to see if FILE_POLL_CONFIG is set, which means there is a config file to be had instead of environmental variables
    let running_config: Config = match env::var("FILE_POLL_CONFIG") {
        Ok(config_file) => Config::unpack_config_file(&config_file),
        Err(_) => Config::parse_env().unwrap(),
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
    // This String will need to be updated as OpenWeatherMaps makes updates/changes to their API endpoints
    let running_url: String = format!("http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}", &running_coords[0], &running_coords[1], running_config.get_key());

    let mut error_count: u8 = 0;
    // This while loop will keep going forever until we hit our error limit
    while error_count < running_config.get_maxretry() {
        let response: Result<PollResponse, ureq::Error> = match get_pollution(&running_url) {
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        };
        // If the response is not an error, unwrap and format it to be placed in the DB then sleep for the set time
        if response.is_ok() {
            let unpacked: PollResponse = response.unwrap();
            let results: PollUpdate = unpacked.unpack();

            write_to_db(&running_client, results, &running_config.get_location()).await?;

            println!("Successfully written to DB {}", running_config.get_dbname());
            // Reset error count if we've had a success
            error_count = 0;
            thread::sleep(Duration::from_secs(running_config.get_timing()));
        } else {
            // If the response is anything but Ok, tick the error count up by one and try to print the error out for later troubleshooting
            println!("Error encountered while grabbing stats.");
            error_count = error_count + 1;
            match response.unwrap_err() {
                ureq::Error::Status(code, resp) => println!("Status: {}, Text: {}", code, resp.status_text()),
                ureq::Error::Transport(trans) => println!("Kind: {}, Message: {}", trans.kind(), trans.message().unwrap_or("N/A")),
            };
            // If we are at our error limit, there is no point in continuing
            if running_config.get_maxretry() <= error_count {
                break;
            } else {
                // If we are under our error limit, sleep for half of the normal time and then run the loop again
                thread::sleep(Duration::from_secs(running_config.get_timing() / 2));
            };
        } 
    }
    // If we make it out of the while loop, we have are at our limit and need to terminate
    panic!("Max errors reached! Terminating loop and script.");
}