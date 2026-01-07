use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, author, about)]
/// A dummy LNURL pay server for testing purposes.
pub struct Config {
    /// Bind address for the webserver
    #[clap(default_value_t = String::from("0.0.0.0"), long, env = "LNURL_BIND")]
    pub bind: String,

    /// Port for the webserver
    #[clap(default_value_t = 3000, long, env = "LNURL_PORT")]
    pub port: u16,

    /// The domain name you are running lnurl-server on
    #[clap(long, env = "LNURL_DOMAIN")]
    pub domain: String,

    /// Default minimum amount in millisatoshis (used when username is not numeric)
    #[clap(default_value_t = 1_000, long, env = "LNURL_MIN_SENDABLE")]
    pub min_sendable: u64,

    /// Default maximum amount in millisatoshis (used when username is not numeric)
    #[clap(default_value_t = 11_000_000_000, long, env = "LNURL_MAX_SENDABLE")]
    pub max_sendable: u64,
}
