use crate::{
    args::{Daemon, Lmtp, Webhook},
    cmd::{
        aliases::cmd as aliases, lmtp::cmd as lmtp, transport::cmd as transport,
        webhook::cmd as webhook,
    },
    config::Config,
    utils::s6_ready,
};
use anyhow::Result;
use std::thread;

pub fn cmd(config: Config, args: Daemon, verbose: bool, debug: bool) -> Result<Option<String>> {
    // get aliases
    aliases(&config)?;
    // get transport
    transport(&config)?;

    // launch webhook and lmtp
    let webhook_args = Webhook {
        port: args.port,
        prefix: args.prefix,
        ready_fd: None,
    };
    let lmtp_args = Lmtp { ready_fd: None };
    let wconfig = config.clone();
    let webhook = thread::spawn(move || webhook(wconfig, webhook_args, verbose));
    let lmtp = thread::spawn(move || lmtp(config, lmtp_args, verbose, debug));

    // s6 readiness notification
    s6_ready(args.ready_fd);

    // wait for threads to finish
    if let Err(_) = webhook.join() {
        eprintln!("webhook error");
    };
    if let Err(_) = lmtp.join() {
        eprintln!("lmtp error");
    };
    Ok(None)
}
