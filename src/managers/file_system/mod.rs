pub mod error;
use error::Error;

use dirs::home_dir;
use std::fs::read_dir;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FileSystemManager {
    data_root_path: PathBuf,
}

impl FileSystemManager {
    pub fn new() -> Result<Self, Error> {
        let home_dir_path = match home_dir() {
            None => Err(Error::HomeDirectoryNotFound),
            Some(home_directory) => Ok(home_directory),
        }?;
        let data_root_path = home_dir_path.clone().join(".mojo").join("data");

        tracing::info!(
            "file system manager initialised at path {:?}",
            home_dir_path
        );

        Ok(Self { data_root_path })
    }

    pub fn get_directory_names_in_directory(
        &self,
        relative_path: String,
    ) -> Result<Vec<String>, Error> {
        let folder_path = self.data_root_path.join(relative_path.clone());
        let mut directories: Vec<String> = vec![];
        for path in read_dir(folder_path.join(relative_path))? {
            let path = path?;
            if path.metadata()?.is_dir() {
                directories.push(
                    folder_path
                        .join(path.file_name())
                        .into_os_string()
                        .into_string()
                        .map_err(|_| Error::Conversion)?,
                )
            }
        }
        Ok(directories)
    }

    pub fn get_file_names_in_directory(&self, relative_path: String) -> Result<Vec<String>, Error> {
        let folder_path = self.data_root_path.join(relative_path.clone());
        let mut directories: Vec<String> = vec![];
        for path in read_dir(folder_path.join(relative_path))? {
            let path = path?;
            if path.metadata()?.is_file() {
                directories.push(
                    folder_path
                        .join(path.file_name())
                        .into_os_string()
                        .into_string()
                        .map_err(|_| Error::Conversion)?,
                )
            }
        }
        Ok(directories)
    }
}
