use auth::{App, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = App::new().run().await {
        eprintln!("An error occurred: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
