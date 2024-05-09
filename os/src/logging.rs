use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct SimpleLogger;

impl Log for SimpleLogger {
    //是否应该记录具有给定元数据的日志记录
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    //日志记录逻辑
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, //red
            Level::Warn => 93,  //yellow
            Level::Info => 34,  //blue
            Level::Debug => 32, //green
            Level::Trace => 90, //BrightBlack
        };
        //\x1b[31mhello world\x1b[0m
        println!(
            "\u{1B}[{}m[{:>5}]{}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    //刷新缓存
    fn flush(&self) {}
}

//初始化日志系统
pub fn init() {
    //静态变量来存储日志记录器:任何地方都可以访问
    static LOGGER: SimpleLogger = SimpleLogger;
    //设置为全局的日志记录器
    log::set_logger(&LOGGER).unwrap();
    //从环境变量 LOG 获取值来设置最大的日志级别
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}