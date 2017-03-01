extern crate docopt;
extern crate rustc_serialize;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate reqwest;
extern crate mime;

use docopt::Docopt;
use reqwest::Client;
use reqwest::header::{Headers, Accept, qitem, Authorization, Basic};
use mime::{Mime, Attr, SubLevel, TopLevel, Value};
use std::io::Read;

const USAGE: &'static str = "
sumr
Usage:
  sumr [options] <base-url> <build-id> <username> <password>
  sumr -h | --help
Options:
  -h --help                         Show this screen.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_username: String,
    arg_password: String,
    arg_base_url: String,
    arg_build_id: String,
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

    let client = Client::new().unwrap();

    let builds_ids = get_dependent_builds(&client,
                                          &args.arg_base_url,
                                          &args.arg_build_id,
                                          &args.arg_username,
                                          &args.arg_password)
        .unwrap();

    let mut total_build_time = 0;

    for id in builds_ids {
        total_build_time += get_build_time(&client,
                                           &args.arg_base_url,
                                           &id.to_string(),
                                           &args.arg_username,
                                           &args.arg_password)
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
