use std::{fs::File, path::PathBuf};

use anyhow::Context;

const PROJECT_NAMESPACE: &str = "museun";
const PROJECT_NAME: &str = "readchat";

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as _
}

#[derive(Default)]
pub struct Logger {
    transcribe: Target,
}

impl Logger {
    pub fn get_dir() -> anyhow::Result<PathBuf> {
        dirs::data_local_dir()
            .map(|base| base.join(PROJECT_NAMESPACE).join(PROJECT_NAME))
            .with_context(|| "a local data directory must exist on your system")
    }

    pub fn from_xdg(channel: &str) -> anyhow::Result<Self> {
        let dir = Self::get_dir()?;
        std::fs::create_dir_all(&dir)?;

        std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(dir.join(channel).with_extension("log"))
            .map(Target::File)
            .map(|transcribe| Self { transcribe })
            .map_err(Into::into)
    }

    pub fn transcribe(&mut self, msg: &str) -> anyhow::Result<()> {
        use std::io::Write as _;
        if let Target::File(file) = &mut self.transcribe {
            writeln!(file, "{}", msg)?;
            file.sync_all()?;
        }
        Ok(())
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.transcribe(&format!("*** session end: {}", timestamp()));
    }
}

pub enum Target {
    File(File),
    Noop,
}

impl Default for Target {
    fn default() -> Self {
        Self::Noop
    }
}
