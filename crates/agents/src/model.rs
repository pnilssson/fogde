mod claude;
mod codex;
mod copilot;
mod tail;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use protocol::agents::Agent;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

const SLACK: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
struct Process {
    pid: u32,
    started: SystemTime,
}

impl Process {
    fn before(self, time: SystemTime) -> bool {
        time + SLACK >= self.started
    }
}

struct Running {
    processes: Vec<Process>,
    started: Option<SystemTime>,
}

pub async fn model(agent: Agent, foreground: Vec<u32>) -> Option<String> {
    if foreground.is_empty() {
        return None;
    }

    let running = blocking(move || Some(running(&foreground))).await?;
    match agent {
        Agent::Claude => {
            let home = home("CLAUDE_CONFIG_DIR", ".claude")?;
            blocking(move || claude::model(&home, &running.processes)).await
        }
        Agent::Codex => {
            let pids: Vec<u32> = running
                .processes
                .iter()
                .map(|process| process.pid)
                .collect();
            codex::model(&home("CODEX_HOME", ".codex")?, &pids, running.started?).await
        }
        Agent::Copilot => {
            let home = home("COPILOT_HOME", ".copilot")?;
            blocking(move || copilot::model(&home, &running.processes)).await
        }
    }
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Option<T> + Send + 'static,
) -> Option<T> {
    tokio::task::spawn_blocking(work)
        .await
        .inspect_err(|error| tracing::warn!(%error, "could not read the agent's model"))
        .ok()
        .flatten()
}

fn home(variable: &str, folder: &str) -> Option<PathBuf> {
    env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(folder)))
}

fn running(foreground: &[u32]) -> Running {
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());

    let mut pids = foreground.to_vec();
    let mut seen: HashSet<u32> = pids.iter().copied().collect();
    let mut index = 0;
    while let Some(&parent) = pids.get(index) {
        for (pid, process) in system.processes() {
            if process.parent() == Some(Pid::from_u32(parent)) && seen.insert(pid.as_u32()) {
                pids.push(pid.as_u32());
            }
        }
        index += 1;
    }

    let started = foreground
        .iter()
        .filter_map(|&pid| start(&system, pid))
        .min();
    let processes = pids
        .into_iter()
        .filter_map(|pid| {
            Some(Process {
                pid,
                started: start(&system, pid)?,
            })
        })
        .collect();

    Running { processes, started }
}

fn start(system: &System, pid: u32) -> Option<SystemTime> {
    let process = system.process(Pid::from_u32(pid))?;

    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(process.start_time()))
}
