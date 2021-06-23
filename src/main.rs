mod args;
mod cmd;
mod config;
mod errors;
mod utils;

use crate::{
    args::{Opts, SubCommand},
    cmd::{
        aliases::cmd as aliases, daemon::cmd as daemon, lmtp::cmd as lmtp, pipe::cmd as pipe,
        transport::cmd as transport, webhook::cmd as webhook,
    },
    config::get_config,
};
use anyhow::Result;

fn main() {
    match try_main() {
        // simple error output: postfix expect the error code at the beginning of reply (4xx)
        Err(err) => {
            eprintln!("{}", err);
            //err.chain().skip(1).for_each(|cause| eprintln!("{}", cause));
            std::process::exit(1);
        }
        Ok(ret) => {
            if let Some(msg) = ret {
                println!("100 {}", msg);
            }
        }
    }
}

fn try_main() -> Result<Option<String>> {
    let opts: Opts = argh::from_env();
    // get config value in a struct
    let config = get_config(&opts.config)?;

    match opts.subcmd {
        // in get mode extract archive to specified directory
        SubCommand::Pipe(_args) => pipe(&config),
        SubCommand::Aliases(_) => aliases(&config),
        SubCommand::Webhook(args) => webhook(config, args, opts.verbose),
        SubCommand::Lmtp(args) => lmtp(config, args, opts.verbose, opts.debug),
        SubCommand::Daemon(args) => daemon(config, args, opts.verbose, opts.debug),
        SubCommand::Transport(_) => transport(&config),
    }
}
