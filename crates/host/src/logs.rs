use std::io;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn log(name: &str, console: bool) -> io::Result<()> {
    let (file, error) = match appender(name) {
        Ok(file) => (Some(fmt::layer().with_ansi(false).with_writer(file)), None),
        Err(error) => (None, Some(error)),
    };
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(filter)
        .with(file)
        .with((console || error.is_some()).then(fmt::layer))
        .try_init()
        .map_err(io::Error::other)?;

    error.map_or(Ok(()), Err)
}

fn appender(name: &str) -> io::Result<RollingFileAppender> {
    RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(name)
        .filename_suffix("log")
        .max_log_files(7)
        .build(crate::shared::fogde()?.join(".app").join("logs"))
        .map_err(io::Error::other)
}
