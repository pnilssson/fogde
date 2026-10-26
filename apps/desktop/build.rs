fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows") {
        winresource::WindowsResource::new()
            .set_icon("icons/icon.ico")
            .compile()?;
    }

    Ok(())
}
