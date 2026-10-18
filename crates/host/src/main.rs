#[cfg(unix)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args[..] {
        ["serve", "--build", build] => {
            logs()?;
            host::serve(build.to_owned()).await
        }
        ["proxy", "--build", build] => host::proxy(build).await,
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: fogde-host serve|proxy --build <id>",
        )),
    }
}

#[cfg(unix)]
fn logs() -> std::io::Result<()> {
    let dir = host::fogde()?.join(".app").join("logs");
    std::fs::create_dir_all(&dir)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("host.log"))?;

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(file))
        .init();

    Ok(())
}

#[cfg(not(unix))]
fn main() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "fogde-host runs on Linux servers; the desktop app embeds the host",
    ))
}
