// Distributed under the OSI-approved BSD 2-Clause License.
// See accompanying LICENSE file for details.

use std::borrow::Borrow;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

use crates::directories::ProjectDirs;

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
        let appname_osstr = exe_path
            .file_name()
            .expect("there should be a filename on the executable");
        let appname = appname_osstr.to_string_lossy();

        let project_dirs = ProjectDirs::from("", "", appname.borrow())
            .expect("failed to create project directories");

        Ok(Paths {
            config_dir: project_dirs.data_local_dir().join("data"),
            data_dir: project_dirs.config_dir().to_path_buf(),
        })
    }

    /// Return the base path for the installation.
    fn base_path_dir(source_path: &Path) -> io::Result<(PathBuf, bool)> {
        let mut exe_path = env::current_exe()?;

        exe_path.pop(); // build config (build) or bin (install)

        if "bin"
            == exe_path
                .file_name()
                .expect("the executable path should have a parent directory")
        {
            // In an install tree.
            exe_path.pop(); // install root

            Ok((exe_path, true))
        } else {
            Ok((source_path.to_path_buf(), false))
        }
    }
}
