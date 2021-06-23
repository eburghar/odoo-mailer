use crate::{args::Lmtp, config::Config, utils::s6_ready};
use anyhow::Result;
use bufstream::BufStream;
use std::{
    fs::{remove_file, set_permissions, File, Permissions},
    io::{BufRead, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};
use ureq::post;

#[macro_export]
macro_rules! return_on_err(
    ($inp:expr) => {
        if $inp.is_err() {
            return;
        }
    }
);

static OK: &'static str = "250 OK\r\n";

struct Context {
    data: String,
    quit: bool,
    crlf: bool,
}

impl Context {
    fn deliver(&self, config: &Config) -> String {
        let url = format!("https://{}/mail_delivery/pipe", &config.host);
        let resp = post(&url)
            .timeout_connect(10_000)
            .set("X-Mail-Token", &config.token)
            .set("Content-Type", "text/plain")
            .send_string(&self.data);

        if resp.ok() {
            OK.to_string()
        } else {
            let code = resp.status();
            let msg = match resp.into_string() {
                Ok(msg) => msg,
                _ => "".to_string(),
            };
            format!("421 {} ({})\r\n", msg, code)
        }
    }
}

fn handle_client(stream: UnixStream, config: Arc<Config>, verbose: bool, debug: bool) {
    let _ = stream.set_read_timeout(Some(Duration::new(5, 0)));
    let mut stream = BufStream::new(stream);
    let mut l = Context {
        data: String::new(),
        quit: false,
        crlf: false,
    };
    return_on_err!(stream.write(b"220 localhost LMTP server ready\r\n"));
    return_on_err!(stream.flush());
    loop {
        let mut command = String::new();
        match stream.read_line(&mut command) {
            Ok(_) => {
                if command.is_empty() {
                    return;
                }
                let trimmed_command = (&command[..]).trim();
                let mut args = trimmed_command.split(' ');
                let invalid = "500 Invalid command\r\n".to_string();
                let data_res = b"354 Start mail input; end with <CRLF>.<CRLF>\r\n";
                let ok = OK.to_string();
                let res = match args.next() {
                    Some(cmd) => {
                        if verbose {
                            eprintln!("{}", trimmed_command);
                        }
                        match &cmd.to_ascii_lowercase()[..] {
                            "lhlo" => match args.next() {
                                Some(domain) => format!("250 {}\r\n", domain),
                                _ => invalid,
                            },
                            "rset" | "noop" | "mail" | "rcpt" => ok,
                            "quit" => {
                                l.quit = true;
                                "221 localhost Closing connection\r\n".to_string()
                            }
                            "vrfy" => invalid,
                            "data" => {
                                return_on_err!(stream.write(data_res));
                                return_on_err!(stream.flush());
                                let mut res = invalid;
                                loop {
                                    let mut line = String::new();
                                    match stream.read_line(&mut line) {
                                        Ok(_) => {
                                            if line.is_empty() {
                                                break;
                                            }
                                            if l.crlf && line == ".\r\n" {
                                                res = l.deliver(&config);
                                                l.data = String::new();
                                                break;
                                            } else {
                                                if line.ends_with("\r\n") {
                                                    l.crlf = true
                                                } else {
                                                    l.crlf = false
                                                }
                                                l.data.push_str(&line);
                                            }
                                        }
                                        // EOF
                                        _ => {
                                            // write partial data to /tmp for debuging purpose
                                            if debug && l.data.len() > 0 {
                                                let time = SystemTime::now()
                                                    .duration_since(SystemTime::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs();
                                                let mut file =
                                                    File::create(format!("/tmp/lmtp_{}", time))
                                                        .unwrap();
                                                let _ = file.write(l.data.as_bytes());
                                            }
                                            break;
                                        }
                                    }
                                }
                                res
                            }
                            _ => invalid,
                        }
                    }
                    None => invalid,
                };
                return_on_err!(stream.write(res.as_bytes()));
                return_on_err!(stream.flush());
                if l.quit {
                    return;
                }
            }
            _ => {
                break;
            }
        }
    }
}

pub fn cmd(config: Config, args: Lmtp, verbose: bool, debug: bool) -> Result<Option<String>> {
    let _ = remove_file(&config.socket);
    let listener = UnixListener::bind(&config.socket)?;
    let permissions = Permissions::from_mode(0o666);
    set_permissions(&config.socket, permissions)?;

    println!("lmtp serving at {}", config.socket);

    // s6 readiness notification
    s6_ready(args.ready_fd);

    let config = Arc::new(config);
    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let aconfig = config.clone();
                thread::spawn(move || handle_client(stream, aconfig, verbose, debug));
            }
            Err(_err) => {
                /* connection failed */
                break;
            }
        }
    }
    Ok(None)
}
