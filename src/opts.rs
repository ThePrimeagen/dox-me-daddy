use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct EventOpts {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// The port to use for the events to be served on
    #[structopt(short = "p", long = "port", default_value = "42069")]
    pub port: u16,

    /// The address to use.  Should be 0.0.0.0
    #[structopt(short = "a", long = "addr", default_value = "0.0.0.0")]
    pub addr: String,
}
