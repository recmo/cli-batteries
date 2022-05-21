#[derive(StructOpt)]
struct Options {
    #[structopt(flatten)]
    log: crate::logging::Options,

    #[structopt(flatten)]
    app: crate::Options,
}

pub fn main() -> EyreResult<()> {
    // Install error handler
    color_eyre::install()?;

    // Parse CLI and handle help and version (which will stop the application).
    let matches = Options::clap().long_version(VERSION).get_matches();
    let options = Options::from_clap(&matches);

    // Start log system
    options.log.init()?;

    // TODO: Initialize `rand` and `rayon`.

    // Launch Tokio runtime
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .wrap_err("Error creating Tokio runtime")?
        .block_on(async {
            // Monitor for Ctrl-C
            shutdown::watch_signals();

            // Start main
            crate::async_main(options.app).await?;

            // Initiate shutdown if main returns
            shutdown::shutdown();

            Result::<(), Error>::Ok(())
        })?;

    // Terminate successfully
    debug!("Program terminating normally");
    Ok(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
