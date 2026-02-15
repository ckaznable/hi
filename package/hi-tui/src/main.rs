use anyhow::Result;

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    hi_tui::run_tui(None).await
}
