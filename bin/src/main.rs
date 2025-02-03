use chrono;
use clap::Parser;
use env_logger;

#[derive(Parser, Debug)]
#[clap(name = "miax-agent")]
struct Cli {
    #[clap(long, short)]
    config: bool,
}

fn log_init() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "{} [{}] - {} - {} - {}:{}",
            chrono::Utc::now().to_rfc3339(),
            record.level(),
            record.target(),
            record.args(),
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
        )
    });
    builder.init();
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    log_init();
    let cli = Cli::parse();

    let options = agent::cli::AgentOptions {
        config: cli.config,
        command: None,
    };

    let _ = agent::run(false, &options);
}
