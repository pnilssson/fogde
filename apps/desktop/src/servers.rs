#[cfg(debug_assertions)]
pub fn binaries() -> host::Binaries {
    host::Binaries::built(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"))
}

#[cfg(not(debug_assertions))]
pub fn binaries() -> host::Binaries {
    host::Binaries::embedded(&[
        (
            "x86_64-unknown-linux-musl",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/x86_64-unknown-linux-musl/release/fogde-host"
            )),
        ),
        (
            "aarch64-unknown-linux-musl",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/aarch64-unknown-linux-musl/release/fogde-host"
            )),
        ),
    ])
}
