use slog::Drain;
use slog::Logger;
use slog_async::Async;
use slog_term::FullFormat;
use slog_term::TermDecorator;

pub fn setup() -> Logger {
    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator).build().fuse();
    let drain = Async::new(drain).build().fuse();

    let log = Logger::root(drain, o!());
    log
}