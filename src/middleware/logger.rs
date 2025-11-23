use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};
use std::fs;

pub fn init_file_logger() -> tracing_appender::non_blocking::WorkerGuard {
    let directory = "logs";
    fs::create_dir_all(directory).unwrap();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, directory, "requests.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = Registry::default()
        .with(fmt::Layer::default().with_writer(non_blocking).with_ansi(false));

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    guard
}
