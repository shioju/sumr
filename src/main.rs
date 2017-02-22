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
use rustc_serialize::json;
use std::io::Read;

const USAGE: &'static str = "
sumr
Usage:
  sumr [options] <username> <password> <base_url> <build_id>
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

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);
    let url = args.arg_base_url + "/app/rest/latest/builds?locator=snapshotDependency:(to:(id:" + &args.arg_build_id + "),includeInitial:true),defaultFilter:false";
    let mut response = get(&*url,
                       &*args.arg_username,
                       &*args.arg_password).map_err(|err| err.to_string()).unwrap();

    let mut json_string = String::new();
    response.read_to_string(&mut json_string).map_err(|err| err.to_string()).unwrap();
    println!("{}", json_string);
}

fn get(url: &str, username: &str, password: &str) -> Result<reqwest::Response, reqwest::Error> {
  let mut headers = Headers::new();
  headers.set(
    Accept(vec![
        qitem(Mime(TopLevel::Application, SubLevel::Json,
                   vec![(Attr::Charset, Value::Utf8)])),
    ])
  );
  headers.set(
     Authorization(
         Basic {
             username: username.to_string(),
             password: Some(password.to_string())
         }
     )
  );
  let client = Client::new()?;
  let request_builder = client.get(url);
  let request_builder = request_builder.headers(headers);

  let result = request_builder.send();
  return result;
}
