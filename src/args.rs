use argh::FromArgs;

#[derive(FromArgs)]
/// Send email to an Odoo server
pub struct Opts {
    #[argh(
        option,
        short = 'c',
        default = "\"/etc/postfix/odoo-mailer.yml\".to_string()"
    )]
    /// configuration file containing connection parameters
    pub config: String,
    #[argh(switch, short = 'v')]
    /// more detailed output
    pub verbose: bool,
    #[argh(switch, short = 'd')]
    /// debug (dump interupted mail sending)
    pub debug: bool,
    #[argh(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum SubCommand {
    Pipe(Pipe),
    Aliases(Aliases),
    Webhook(Webhook),
    Lmtp(Lmtp),
    Daemon(Daemon),
    Transport(Transport),
}

#[derive(FromArgs)]
/// Send email from stdin
#[argh(subcommand, name = "pipe")]
pub struct Pipe {}

#[derive(FromArgs)]
/// Print aliases
#[argh(subcommand, name = "aliases")]
pub struct Aliases {}

#[derive(FromArgs)]
/// Refresh aliases from a webhook
#[argh(subcommand, name = "webhook")]
pub struct Webhook {
    #[argh(option, short = 'n', default = "8000")]
    /// port to serve webhook from
    pub port: u16,
    #[argh(option, short = 'p', default = "\"/aliases\".to_string()")]
    /// prefix for the webhook
    pub prefix: String,
    #[argh(option, short = 'r')]
    /// readiness file descriptor
    pub ready_fd: Option<i32>,
}

#[derive(FromArgs)]
/// Generate aliases file
#[argh(subcommand, name = "lmtp")]
pub struct Lmtp {
    #[argh(option, short = 'r')]
    /// readiness file descriptor
    pub ready_fd: Option<i32>,
}

// TODO: DRY using trait ?
#[derive(FromArgs)]
/// Daemon mode (lmtp + webhook)
#[argh(subcommand, name = "daemon")]
pub struct Daemon {
    #[argh(option, short = 'n', default = "8000")]
    /// port to serve webhook from
    pub port: u16,
    #[argh(option, short = 'p', default = "\"/aliases\".to_string()")]
    /// prefix for the webhook
    pub prefix: String,
    #[argh(option, short = 'r')]
    /// readiness file descriptor
    pub ready_fd: Option<i32>,
}

#[derive(FromArgs)]
/// Generate transport file
#[argh(subcommand, name = "transport")]
pub struct Transport {}
