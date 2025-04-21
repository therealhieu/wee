use std::sync::OnceLock;

static TRACING: OnceLock<()> = OnceLock::new();

pub fn init_tracing() {
    TRACING.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(
                "shorten=debug,test=debug,info",
            ))
            .try_init();
    });
}
