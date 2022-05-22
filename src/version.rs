#[derive(Clone, Copy, Debug)]
pub struct Version {
    pub pkg_name:     &'static str,
    pub pkg_version:  &'static str,
    pub crate_name:   &'static str,
    pub commit_hash:  &'static str,
    pub long_version: &'static str,
    pub target:       &'static str,
}

#[macro_export]
macro_rules! version {
    () => {
        $crate::Version {
            pkg_name:     env!("CARGO_PKG_NAME"),
            pkg_version:  env!("CARGO_PKG_VERSION"),
            crate_name:   env!("CARGO_CRATE_NAME"),
            commit_hash:  env!("COMMIT_SHA"),
            target:       env!("TARGET"),
            long_version: concat!(
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
            ),
        }
    };
}
