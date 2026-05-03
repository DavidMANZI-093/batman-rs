use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "batman",
    version,
    about = "batman connects directly to the Linux kernel via AF_NETLINK to monitor hardware power events and execute user-defined thresholds."
)]
pub struct Cli {
    /// Path to a custom configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,
}
