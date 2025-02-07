use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct AgentOptions {
    #[clap(long)]
    pub config: bool,

    #[clap(subcommand)]
    pub command: Option<AgentCommands>,
}

#[derive(Parser, Debug)]
pub enum AgentCommands {
    Did {},
}
