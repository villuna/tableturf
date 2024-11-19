use tableturf_server::run;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let address = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:2611".to_owned());

    run(&address).await
}
