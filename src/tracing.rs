pub fn init() {
    use tracing_subscriber::{fmt, EnvFilter};

    // RUST_LOG examples:
    //   RUST_LOG=info
    //   RUST_LOG=kube_podlog=debug,kube=warn
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,kube=warn,kube_runtime=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
