use std::fs;
use {log, fern, time};

pub fn init_logger() {
    let log_path = format!("data/logs/main/OpenStratos.{}.log",
                           time::now_utc()
                               .strftime("%F.%H-%M-%S")
                               .unwrap());

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[OpenStratos][{}] - {} - {}",
                    level,
                    time::now_utc().strftime("%D %T.%f").unwrap(),
                    msg)
        }),
        output: if cfg!(feature = "debug") {
            vec![fern::OutputConfig::stdout(), fern::OutputConfig::file(&log_path)]
        } else {
            vec![fern::OutputConfig::file(&log_path)]
        },
        level: if cfg!(feature = "debug") {
            log::LogLevelFilter::Trace
        } else {
            log::LogLevelFilter::Info
        },
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}

pub fn check_or_create(path: &str) {
    if !fs::metadata(path).is_ok() {
        fs::create_dir(path).unwrap()
    }
}
