use clap::Parser;
use log::info;
use transactify::{state::InfailableState, util::read_all_records};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg()]
    input_path: String,
    #[arg()]
    output_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    info!("Starting app - prasing arguments");
    let args = Args::parse();
    info!("Arguments OK. Processing inputs");
    let mut state = InfailableState::new();
    state.process_transactions(read_all_records(args.input_path).await?.into_iter());
    info!("Done processing. Writing to file.");
    state.store_to_file(args.output_path).await?;
    info!("All done. Goodbye!");
    Ok(())
}
