use std::fmt;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub fn init(json: bool, default_filter: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE);

    if json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}

pub fn trace_enabled() -> bool {
    tracing::enabled!(Level::TRACE)
}

#[macro_export]
macro_rules! lazy_trace {
    ($make:expr) => {{
        if $crate::logging::trace_enabled() {
            tracing::trace!("{}", $make());
        }
    }};
}

pub struct LazyValue<F>(pub F);

impl<F, T> fmt::Display for LazyValue<F>
where
    F: Fn() -> T,
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (self.0)())
    }
}

#[cfg(test)]
mod tests {
    use super::LazyValue;
    use std::cell::Cell;

    #[test]
    fn lazy_value_evaluates_only_when_formatted() {
        let calls = Cell::new(0);
        let value = LazyValue(|| {
            calls.set(calls.get() + 1);
            "expensive"
        });

        assert_eq!(calls.get(), 0);
        assert_eq!(format!("{value}"), "expensive");
        assert_eq!(calls.get(), 1);
    }
}
