// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use std::borrow::Borrow;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

/// Paths for configuration and data storage.
pub struct Paths {
    /// Directory for storing configuration files.
    pub config_dir: PathBuf,
    /// Directory for storing data files.
    pub data_dir: PathBuf,
}

impl Paths {
    /// Construct paths based on a given source tree.
    ///
    /// This allows a binary to be run in both an install tree and a build tree.
    pub fn new<P: AsRef<Path>>(source_path: P) -> io::Result<Self> {
        let (base_dir, is_install) = Self::base_path_dir(source_path.as_ref())?;

        if is_install {
            Self::from_install()
        } else {
            Self::from_build(base_dir)
        }
    }

    /// Paths based on the build directory.
    fn from_build(path: PathBuf) -> io::Result<Self> {
        Ok(Paths {
            config_dir: path.clone(),
            data_dir: path.clone(),
        })
    }

    /// Paths based on the install directory.
    fn from_install() -> io::Result<Self> {
        let exe_path = env::current_exe()?;
        let appname_osstr = exe_path.file_name()
            .expect("there should be a filename on the executable");
        let appname = appname_osstr.to_string_lossy();

        Ok(Paths {
            config_dir: Self::config_dir(appname.borrow()),
            data_dir: Self::data_dir(appname.borrow()),
        })
    }

    #[cfg(windows)]
    fn config_dir(appname: &str) -> PathBuf {
        unimplemented!()
    }

    #[cfg(windows)]
    /// The data directory on Windows.
    fn data_dir(appname: &str) -> PathBuf {
        let mut appdata_dir = PathBuf::from(env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut home = env::home_dir().unwrap();
                home.push("Application Data");
                home
            }));
        appdata_dir.push(appname);
        appdata_dir.push("data");
    }

    #[cfg(not(any(windows)))]
    /// The configuration directory on non-Windows platforms.
    fn config_dir(appname: &str) -> PathBuf {
        let mut config_dir = PathBuf::from(env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut home = env::home_dir()
                    .expect("a home directory is required");
                home.push(".config");
                home
            }));
        config_dir.push(appname);
        config_dir
    }

    #[cfg(not(any(windows)))]
    /// The data directory on non-Windows platforms.
    fn data_dir(appname: &str) -> PathBuf {
        let mut data_dir = PathBuf::from(env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut home = env::home_dir()
                    .expect("a home directory is required");
                home.push(".local");
                home.push("share");
                home
            }));
        data_dir.push(appname);
        data_dir
    }

    /// Return the base path for the installation.
    fn base_path_dir(source_path: &Path) -> io::Result<(PathBuf, bool)> {
        let mut exe_path = env::current_exe()?;

        exe_path.pop(); // build config (build) or bin (install)

        if "bin" == exe_path.file_name()
            .expect("the executable path should have a parent directory") {
            // In an install tree.
            exe_path.pop(); // install root

            Ok((exe_path, true))
        } else {
            Ok((source_path.to_path_buf(), false))
        }
    }
}
