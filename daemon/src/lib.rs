pub mod cpu_miner;
pub mod error;
pub mod imports;
pub mod kaspad;
pub mod result;

use std::fmt::Display;

use crate::imports::*;
pub use crate::result::Result;
pub use cpu_miner::{CpuMiner, CpuMinerConfig, CpuMinerCtl};
pub use kaspad::{Kaspad, KaspadConfig, KaspadCtl};
use workflow_core::runtime;
use workflow_store::fs::*;

pub static LOCATIONS: &[&str] =
    &["bin", "../target/release", "../target/debug", "../../kaspa-miner-cpu/target/debug", "../../kaspa-miner-cpu/target/release"];

pub async fn locate_binaries(root: &str, name: &str) -> Result<Vec<PathBuf>> {
    // log_info!("locating binaries in root: {root} name: {name}");

    if !runtime::is_nw() && !runtime::is_node() && !runtime::is_native() {
        return Err(Error::Platform);
    }

    let name = if runtime::is_windows() { name.to_string() + ".exe" } else { name.to_string() };

    let locations = LOCATIONS
        .iter()
        .map(|path| PathBuf::from(&root).join(path).join(&name).normalize().map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

    let mut list = Vec::new();
    for path in locations {
        log_info!("locating binary: {}", path.display());
        if exists(&path).await? {
            log_info!("found binary: {}", path.display());
            list.push(path);
        } else {
            log_info!("did not find binary: {}", path.display());
        }
    }

    Ok(list)
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum DaemonKind {
    Kaspad,
    CpuMiner,
}

#[derive(Default)]
pub struct Daemons {
    pub kaspad: Option<Arc<dyn KaspadCtl + Send + Sync + 'static>>,
    pub cpu_miner: Option<Arc<dyn CpuMinerCtl + Send + Sync + 'static>>,
}

impl Daemons {
    pub fn new() -> Self {
        Self { kaspad: None, cpu_miner: None }
    }

    pub fn with_kaspad(mut self, kaspad: Arc<dyn KaspadCtl + Send + Sync + 'static>) -> Self {
        self.kaspad = Some(kaspad);
        self
    }

    pub fn with_cpu_miner(mut self, cpu_miner: Arc<dyn CpuMinerCtl + Send + Sync + 'static>) -> Self {
        self.cpu_miner = Some(cpu_miner);
        self
    }

    pub fn kaspad(&self) -> Arc<dyn KaspadCtl + Send + Sync + 'static> {
        self.kaspad.as_ref().expect("accessing Daemons::kaspad while kaspad option is None").clone()
    }

    pub fn cpu_miner(&self) -> Arc<dyn CpuMinerCtl + Send + Sync + 'static> {
        self.cpu_miner.as_ref().expect("accessing Daemons::cpu_miner while cpu_miner option is None").clone()
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Stdio {
    Stdout(DaemonKind, String),
    Stderr(DaemonKind, String),
}

impl From<Stdio> for String {
    fn from(s: Stdio) -> Self {
        match s {
            Stdio::Stdout(_, s) => s,
            Stdio::Stderr(_, s) => s,
        }
    }
}

impl Stdio {
    pub fn kind(&self) -> DaemonKind {
        match self {
            Stdio::Stdout(kind, _) => kind.clone(),
            Stdio::Stderr(kind, _) => kind.clone(),
        }
    }

    pub fn trim(self) -> String {
        let mut s = String::from(self);
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
        s
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub uptime: Option<u64>,
}

impl Display for DaemonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(uptime) = self.uptime {
            write!(f, "running - uptime: {}", format_duration(uptime))?;
        } else {
            write!(f, "not running")?;
        }
        Ok(())
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / (24 * 60 * 60);
    let hours = (seconds / (60 * 60)) % 24;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{0} days {1:02} hours, {2:02} minutes, {3:02} seconds", days, hours, minutes, seconds)
    } else {
        format!("{0:02} hours, {1:02} minutes, {2:02} seconds", hours, minutes, seconds)
    }
}