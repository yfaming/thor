use anyhow::Result;
use std::env::args;
use thor::config::Config;
use thor::http_server::run_http_server;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = match args().skip(1).next() {
        Some(path) => path,
        None => "config.toml".to_string(),
    };
    println!("loading configuration from {}", config_path);
    let config = Config::load_from_toml(config_path.as_ref())?;

    let format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(format.clone())
        .with_writer(std::io::stdout)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let file_appender = tracing_appender::rolling::daily(&config.server.log_dir, "thor.log");
    let (nonblocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .event_format(format.json())
        .with_writer(nonblocking_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);
    tracing::subscriber::set_global_default(subscriber)?;

    run_http_server(&config).await?;
    Ok(())
}
