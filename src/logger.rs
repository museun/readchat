use anyhow::Context as _;

use std::fs::File;
use std::io::Write as _;
use std::path::PathBuf;

const PROJECT_NAMESPACE: &str = "museun";
const PROJECT_NAME: &str = "readchat";

// TODO make this configurable for transisent vs persistant locking of the file
pub struct Logger {
    transcribe: Target,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            transcribe: Target::Noop,
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.transcribe(&format!("*** session end: {}", crate::timestamp()));
    }
}
enum Target {
    File(File),
    Noop,
}

impl Logger {
    pub fn get_dir() -> anyhow::Result<PathBuf> {
        let base = dirs::data_local_dir()
            .with_context(|| "a local data directory must exist on your system")?;

        let dir = base.join(PROJECT_NAMESPACE).join(PROJECT_NAME);
        Ok(dir)
    }

    pub fn from_xdg(channel: &str) -> anyhow::Result<Self> {
        let dir = Self::get_dir()?;
        std::fs::create_dir_all(&dir)?;

        std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(dir.join(format!("{}.log", channel)))
            .map(|fi| Self {
                transcribe: Target::File(fi),
            })
            .map_err(Into::into)
    }

    pub fn transcribe(&mut self, msg: &str) -> anyhow::Result<()> {
        if let Target::File(file) = &mut self.transcribe {
            writeln!(file, "{}", msg)?;
            file.sync_all()?;
        }

        Ok(())
    }
}
