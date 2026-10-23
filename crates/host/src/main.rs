#[cfg(unix)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args[..] {
        ["serve", "--build", build] => {
            if let Err(error) = host::log("host", false) {
                tracing::warn!(%error, "could not open the log file");
            }
            host::serve(build.to_owned()).await
        }
        ["proxy", "--build", build] => host::proxy(build).await,
        ["usage"] => {
            host::report().await;
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: fogde-host serve|proxy --build <id>",
        )),
    }
}

#[cfg(not(unix))]
fn main() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "fogde-host runs on Linux servers; the desktop app embeds the host",
    ))
}
