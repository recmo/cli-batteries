use crate::{logging, shutdown};
use eyre::{Error, Result as EyreResult, WrapErr as _};
use structopt::StructOpt;
use tokio::{runtime, sync::broadcast};
use tracing::debug;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("COMMIT_SHA"),
    " ",
    env!("COMMIT_DATE"),
    "\n",
    env!("TARGET"),
    " ",
    env!("BUILD_DATE"),
    "\n",
    env!("CARGO_PKG_AUTHORS"),
    "\n",
    env!("CARGO_PKG_HOMEPAGE"),
    "\n",
    env!("CARGO_PKG_DESCRIPTION"),
);

