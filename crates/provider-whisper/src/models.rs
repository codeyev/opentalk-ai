//! Whisper model files: which ones the app offers, and which are installed.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// First bytes of every Whisper model file in the `ggml` format.
const MODEL_FILE_MAGIC: &[u8; 4] = b"lmgg";

/// A model the app can download on request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    /// Short name, such as `base`.
    pub id: &'static str,
    pub file_name: &'static str,
    /// Size of the file, and roughly the memory the model needs.
    pub size_bytes: u64,
    /// SHA-256 of the file, checked after every download.
    #[serde(skip)]
    pub sha256: &'static str,
}

/// The multilingual models offered, smallest first. Larger models are more
/// accurate and slower. Sizes and hashes are those published in the
/// whisper.cpp model repository, read on 2026-11-11.
pub const CATALOGUE: [ModelSpec; 4] = [
    ModelSpec {
        id: "tiny",
        file_name: "ggml-tiny.bin",
        size_bytes: 77_691_713,
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
    },
    ModelSpec {
        id: "base",
        file_name: "ggml-base.bin",
        size_bytes: 147_951_465,
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
    },
    ModelSpec {
        id: "small",
        file_name: "ggml-small.bin",
        size_bytes: 487_601_967,
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    },
    ModelSpec {
        id: "large-v3-turbo",
        file_name: "ggml-large-v3-turbo.bin",
        size_bytes: 1_624_555_275,
        sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
    },
];

/// Where the catalogue's files are published.
const DOWNLOAD_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

impl ModelSpec {
    /// The catalogue entry with this id, if there is one.
    #[must_use]
    pub fn find(id: &str) -> Option<Self> {
        CATALOGUE.into_iter().find(|spec| spec.id == id)
    }

    /// The address the file is downloaded from.
    #[must_use]
    pub fn download_url(&self) -> String {
        format!("{DOWNLOAD_BASE}/{}", self.file_name)
    }
}

/// A model file present in the models folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModel {
    pub file_name: String,
    pub size_bytes: u64,
}

/// Whether `file_name` is a plain model file name: `ggml-<name>.bin`, with
/// no folder parts. Names come from the UI, so this is what keeps the app
/// from being pointed at a file outside the models folder.
#[must_use]
pub fn is_model_file_name(file_name: &str) -> bool {
    let name_part = file_name
        .strip_prefix("ggml-")
        .and_then(|rest| rest.strip_suffix(".bin"))
        .unwrap_or_default();
    !name_part.is_empty()
        && name_part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// The path of a model in `directory`, if `file_name` names a model file
/// that exists there and has the right format.
#[must_use]
pub fn installed_path(directory: &Path, file_name: &str) -> Option<PathBuf> {
    let path = directory.join(file_name);
    (is_model_file_name(file_name) && has_model_format(&path)).then_some(path)
}

/// Lists the model files in `directory`, by name. Files the user copied
/// there count just like downloaded ones. A missing folder is empty.
#[must_use]
pub fn installed(directory: &Path) -> Vec<InstalledModel> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut models: Vec<InstalledModel> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name().into_string().ok()?;
            let path = installed_path(directory, &file_name)?;
            Some(InstalledModel {
                size_bytes: fs::metadata(path).ok()?.len(),
                file_name,
            })
        })
        .collect();
    models.sort_by(|first, second| first.file_name.cmp(&second.file_name));
    models
}

/// Whether the file at `path` starts like a Whisper model.
fn has_model_format(path: &Path) -> bool {
    use std::io::Read;
    let mut magic = [0_u8; 4];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .is_ok_and(|()| &magic == MODEL_FILE_MAGIC)
}

#[cfg(test)]
pub(crate) mod testing {
    use std::path::PathBuf;

    /// A fresh, empty folder for one test. Removed when dropped.
    pub(crate) struct TempDir(pub(crate) PathBuf);

    impl TempDir {
        pub(crate) fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("opentalk-test-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("a temporary folder");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::TempDir;
    use super::*;

    #[test]
    fn only_plain_model_file_names_are_accepted() {
        for name in [
            "ggml-base.bin",
            "ggml-large-v3-turbo.bin",
            "ggml-small.en-q5_1.bin",
        ] {
            assert!(is_model_file_name(name), "{name} should be accepted");
        }
        for name in [
            "ggml-.bin",
            "base.bin",
            "ggml-base.txt",
            "../ggml-base.bin",
            "ggml-../../secret.bin",
            "sub/ggml-base.bin",
            "ggml-base.bin/..",
            "",
        ] {
            assert!(!is_model_file_name(name), "{name:?} should be refused");
        }
    }

    #[test]
    fn installed_models_are_the_model_files_in_the_folder() {
        let folder = TempDir::new("installed");
        fs::write(folder.0.join("ggml-base.bin"), b"lmgg-model-data").unwrap();
        fs::write(folder.0.join("ggml-broken.bin"), b"not a model").unwrap();
        fs::write(folder.0.join("notes.txt"), b"lmgg").unwrap();

        assert_eq!(
            installed(&folder.0),
            vec![InstalledModel {
                file_name: "ggml-base.bin".to_owned(),
                size_bytes: 15,
            }]
        );
        assert!(installed_path(&folder.0, "ggml-base.bin").is_some());
        assert!(installed_path(&folder.0, "ggml-broken.bin").is_none());
        assert!(installed_path(&folder.0, "ggml-missing.bin").is_none());
    }

    #[test]
    fn a_missing_folder_has_no_models() {
        assert!(installed(Path::new("/nonexistent/opentalk/models")).is_empty());
    }

    #[test]
    fn the_catalogue_is_consistent() {
        for spec in CATALOGUE {
            assert!(is_model_file_name(spec.file_name));
            assert_eq!(spec.sha256.len(), 64);
            assert_eq!(ModelSpec::find(spec.id), Some(spec));
            assert!(spec.download_url().starts_with("https://"));
        }
        assert_eq!(ModelSpec::find("imaginary"), None);
    }
}
