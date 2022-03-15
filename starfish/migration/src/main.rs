use sea_schema::migration::*;
use starfish_core::migrator::Migrator;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
