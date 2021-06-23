use crate::{
    args::Webhook,
    config::Config,
    utils::{s6_ready, MapType},
};
use anyhow::Result;
use std::collections::hash_map::HashMap;
use tiny_http::{Header, Method, Response, Server};

fn get_header<'a>(headers: &'a [Header], key: &'static str) -> Option<&'a str> {
    headers
        .iter()
        .find(|&header| header.field.equiv(key))
        .and_then(|v| Some(v.value.as_str()))
}

pub fn cmd(config: Config, args: Webhook, verbose: bool) -> Result<Option<String>> {
    let server = Server::http(format!("0.0.0.0:{}", args.port)).unwrap();

    println!(
        "webhook serving at http://0.0.0.0:{}{}",
        args.port, args.prefix
    );

    // s6 readiness notification
    s6_ready(args.ready_fd);

    for mut request in server.incoming_requests() {
        if verbose {
            println!(
                "received request! method: {:?}, url: {:?}, headers: {:?}",
                request.method(),
                request.url(),
                request.headers()
            );
        }
        // check that it's a post request with configured prefix
        if request.method() == &Method::Post && request.url() == args.prefix {
            // check that we have yaml body
            match get_header(request.headers(), &"content-type") {
                Some("application/yaml") => (),
                _ => {
                    eprintln!("error {}", "no encoded yaml");
                    continue;
                }
            }
            // check the token
            match get_header(request.headers(), &"x-mail-token") {
                // authorized
                Some(header) if config.token == header => {
                    let mut data = String::new();
                    request.as_reader().read_to_string(&mut data)?;
                    // serialize the yaml
                    let mut aliases_map = Vec::new();
                    let mut transport_map = Vec::new();
                    match serde_yaml::from_str::<HashMap<String, Vec<String>>>(&data) {
                        Ok(data) => {
                            for (account, aliases) in data {
                                let v: Vec<&str> = account.split('@').collect();
                                let domain = match v.get(1) {
                                    Some(domain) => domain,
                                    None => "",
                                };
                                aliases_map.push(String::from(
                                    &aliases
                                        .iter()
                                        .map(|v| format!("{}@{} {}", v, domain, account))
                                        .collect::<Vec<String>>()
                                        .join("\n"),
                                ));
                                transport_map
                                    .push(format!("{} lmtp:unix:{}", account, &config.socket));
                            }
                        }
                        Err(e) => {
                            eprintln!("error {}", e);
                        }
                    }
                    // write aliases map
                    MapType::Aliases.write(&config, aliases_map.join("\n").as_bytes())?;
                    // write transport map
                    MapType::Transport.write(&config, transport_map.join("\n").as_bytes())?;
                    request.respond(Response::empty(200))?;
                }
                // unauthorized
                _ => {
                    request.respond(Response::empty(401))?;
                }
            };
        // not found
        } else {
            request.respond(Response::empty(404))?;
        }
    }

    Ok(None)
}
