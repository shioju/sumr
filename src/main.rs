extern crate docopt;
extern crate rustc_serialize;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate reqwest;
extern crate mime;
extern crate toml;

use docopt::Docopt;
use reqwest::Client;
use reqwest::header::{Headers, Accept, qitem, Authorization, Basic};
use mime::{Mime, Attr, SubLevel, TopLevel, Value};
use std::io::Read;
use std::fs::File;

const USAGE: &'static str = "
sumr
Usage:
  sumr [options] <configuration-file>
  sumr -h | --help
Options:
  -h --help                         Show this screen.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_configuration_file: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Config {
    username: String,
    password: String,
    base_url: String,
    build_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Builds {
    count: u32,
    href: String,
    build: Vec<Build>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Build {
    id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildStatistics {
    property: Vec<BuildProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildProperties {
    name: String,
    value: u32,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let config = read_config(&args.arg_configuration_file)
        .map_err(|err| {
            panic!("failed to read configuration file {}: {}",
                   &args.arg_configuration_file,
                   err)
        })
        .unwrap();

    let client = Client::new().unwrap();

    let builds_ids = get_dependent_builds(&client,
                                          &config.base_url,
                                          &config.build_id,
                                          &config.username,
                                          &config.password)
        .unwrap();

    let mut total_build_time = 0;

    for id in builds_ids {
        total_build_time += get_build_time(&client,
                                           &config.base_url,
                                           &id.to_string(),
                                           &config.username,
                                           &config.password)
            .unwrap();
    }

    println!("total build time = {}", total_build_time);
}

fn get_build_time(client: &Client,
                  base_url: &str,
                  build_id: &str,
                  username: &str,
                  password: &str)
                  -> Result<u32, String> {
    let url = base_url.to_string() + "/app/rest/latest/builds/id:" + build_id + "/statistics";

    let mut response = get(client, &*url, username, password).map_err(|err| err.to_string())?;

    let mut json_string = String::new();
    response.read_to_string(&mut json_string).map_err(|err| err.to_string())?;
    let deserialized: BuildStatistics =
        serde_json::from_str(&json_string).map_err(|err| err.to_string())?;

    let build_time: Vec<u32> = deserialized.property
        .iter()
        .filter(|prop| prop.name == "BuildDurationNetTime")
        .map(|prop| prop.value)
        .collect();

    Ok(build_time[0])
}

fn get_dependent_builds(client: &Client,
                        base_url: &str,
                        build_id: &str,
                        username: &str,
                        password: &str)
                        -> Result<Vec<u32>, String> {
    let url = base_url.to_string() + "/app/rest/latest/builds?locator=snapshotDependency:(to:(id:" +
              build_id + "),includeInitial:true),defaultFilter:false";

    let mut response = get(client, &*url, username, password).map_err(|err| err.to_string())?;

    let mut json_string = String::new();
    response.read_to_string(&mut json_string).map_err(|err| err.to_string())?;
    let deserialized: Builds = serde_json::from_str(&json_string).map_err(|err| err.to_string())?;

    let builds = deserialized.build.iter().map(|build| build.id).collect();

    Ok(builds)
}

fn get(client: &Client,
       url: &str,
       username: &str,
       password: &str)
       -> Result<reqwest::Response, reqwest::Error> {
    let mut headers = Headers::new();
    headers.set(Accept(vec![qitem(Mime(TopLevel::Application,
                                       SubLevel::Json,
                                       vec![(Attr::Charset, Value::Utf8)]))]));
    headers.set(Authorization(Basic {
        username: username.to_string(),
        password: Some(password.to_string()),
    }));
    let request_builder = (*client).get(url);
    let request_builder = request_builder.headers(headers);

    request_builder.send()
}

fn read_config(path: &str) -> Result<Config, String> {
    let mut file = File::open(&path).map_err(|err| err.to_string())?;
    let mut config_toml = String::new();
    file.read_to_string(&mut config_toml).map_err(|err| err.to_string())?;

    toml::from_str(&config_toml).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
  use super::{Config, read_config};

  #[test]
  fn it_reads_and_parses_a_config_file() {
    let expected = Config {
        base_url: "https://teamcity.example.com".to_string(),
        build_id: "123".to_string(),
        username: "username".to_string(),
        password: "password".to_string(),
    };

    let actual = read_config("tests/fixtures/config.toml").unwrap();

    assert_eq!(expected, actual);
  }
}
