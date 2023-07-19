use hyperwarphooker::utils::config::Config;

type TokResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn run_with_config(config: Config) -> TokResult<()>{
    // streamerd is run on the same machine as the target
    // thus we will use the unix transport
    if !config.connection_type.to_owned().starts_with("unix") {
        println!("seems like connection type is not unix like. unix socket connection may fail ");
    }

    let socket_path = config.unix_socket_path.expect("please specify unix socket path");
    let socket = tokio::net::UnixStream::connect(socket_path).await?;

    

    Ok(())
}

#[tokio::main]
async fn main() -> TokResult<()> {
    println!("streamer daemon v{}",env!("CARGO_PKG_VERSION"));
    let config = Config::from_env();
    run_with_config(config).await
}