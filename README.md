# pollutionclient_rs
OpenWeatherMaps pollution API Rust Client

# Description
This is a self contained crate designed to run within a container (non-root). When configured appropriately, via a TOML configuration file or environmental variables, it will regularly pull air quality and pollutant information from OpenWeatherMaps pollution API and write it to an InfluxDB that you designate.

# Before you begin
If you don't already have one, create an account with OpenWeatherMaps at https://home.openweathermap.org/users/sign_up <br>
Once signed up, generate an API key and give the system roughly 4 hours to allow your key access.

Create an InfluxDB database with an appropriate name. Create a user or token for that DB that has write permissions (read permissions are not required).

# Recommended Setup
Clone the repository and build the image as you see fit using the included Dockerfile.

Put all your needed options in a text/toml file in a secure location on the host running the container.

When running the container, map the a volume to '/usr/src/pollutionclient_rs/config/<yourconfigfile>' and set the environmental variable "FILE_POLL_CONFIG" to that location.

Example (This command is assuming you have a directory named "config" that contains the my_config.toml file needed by the program. Also that you named the image "pollutionclient_rs:latest"):
```
docker run -d --restart:unless-stopped --env FILE_POLL_CONFIG='/usr/src/pollutionclient_rs/config/my_config.toml' -v ${PWD}/config:/usr/src/pollutionclient_rs/config pollutionclient_rs:latest
```

# Configuration options
The recommended option is to create a TOML file with your configuration options and protect it. The keys for the TOML file are the same as the environmental variable names.

Alternatively, you can generate environmental variables for each needed configuration.

## Required Environmental Variables (if not using a config file)
- OPENWEATHER_API_KEY
  - The API key generated for your account by OpenWeatherMaps
- OPENWEATHER_POLL_ZIP
  - The zipcode where the statistics are desired
- OPENWEATHER_INFLUXDB_NAME
  - The name of the database to write to. Defaults to "test" if not provided.
- OPENWEATHER_INFLUXDB_SERVER
  - The host that will be taking writes of the data. Is expecting "http://" at the start and will add it if it does not see it. If no port is provided, it will add the default "8086"
 
### InfluxDB Server Name Examples
 
Valid
- localhost
- http://localhost
- http://localhost:8080
- https://localhost.example.com
- localhost:8086

Invalid
- tcp://localhost:8080
 
## Optional Environmental Variables
- OPENWEATHER_POLL_TIMING
  - The frequency in seconds to check for pollution (Note, OpenWeatherMaps updates pollution stats hourly and thus the default is 3600)
- OPENWEATHER_MAX_RETRY
  - The maximum failed collections to tolerate. Default is 3. This only handles API errors, not panics from the program.
- OPENWEATHER_POLL_COUNTRY
  - If your zipcode is not within the US. You will need to specify your country in a way that OpenWeatherMaps recognizes via their <a href="https://openweathermap.org/api/geocoding-api">API documentation</a>.
- OPENWEATHER_INFLUXDB_DBUSER
  - The username with write permissions to the outlined database ***must be declared with OPENWEATHER_INFLUXDB_DBPASS***
- OPENWEATHER_INFLUXDB_DBPASS
  - The password for the provided username to the outlined database ***must be declared with OPENWEATHER_INFLUXDB_DBUSER***
- OPENWEATHER_INFLUXDB_TOKEN
  - The token to use to connect to InfluxDB v2 or cloud

# Final Notes
I made this for myself. I'm using it to track pollution in my area and dump the stats into Grafana. If you have questions, feel free to reach out. If you have PRs, those are always welcome.
