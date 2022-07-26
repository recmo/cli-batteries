#[derive(Clone, Debug)]
pub struct Version {
    pub pkg_name:     &'static str,
    pub pkg_version:  &'static str,
    pub pkg_repo:     &'static str,
    pub crate_name:   &'static str,
    pub commit_hash:  &'static str,
    pub long_version: &'static str,
    pub target:       &'static str,
    pub app_crates:   Vec<String>,
}

#[macro_export]
macro_rules! version {
    ($($c:ident),* ) => {
        $crate::Version {
            pkg_name:     env!("CARGO_PKG_NAME"),
            pkg_version:  env!("CARGO_PKG_VERSION"),
            pkg_repo:     env!("CARGO_PKG_REPOSITORY"),
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
            app_crates:   vec![
                env!("CARGO_PKG_NAME").replace('-', "_"),
                env!("CARGO_CRATE_NAME").replace('-', "_"),
                "cli_batteries".to_owned(),
                $(
                    stringify!($c).to_string(),
                )*
            ],
        }
    };
}
