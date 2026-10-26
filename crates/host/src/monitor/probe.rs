use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use protocol::monitor::{Battery, Disk, Load, Memory, Sample};
use starship_battery::units::energy::watt_hour;
use starship_battery::units::power::watt;
use starship_battery::{Manager, State};
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate,
    RefreshKind, System, UpdateKind,
};

pub struct Shell {
    pub terminal: String,
    pub pid: u32,
}

pub struct Probe {
    system: System,
    disks: Disks,
}

struct Usage {
    cpu: f64,
    memory: u64,
}

impl Probe {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, processes());

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn sample(
        &mut self,
        shells: &[Shell],
        herdr: Option<&Path>,
        repos: Option<&Path>,
    ) -> Sample {
        self.system.refresh_cpu_usage();
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        self.system
            .refresh_processes_specifics(ProcessesToUpdate::All, true, processes());
        self.disks.refresh(true);

        let children = self.children();
        let mut roots = vec![Pid::from_u32(std::process::id())];
        roots.extend(shells.iter().map(|shell| Pid::from_u32(shell.pid)));
        if let Some(herdr) = herdr {
            roots.extend(
                self.system
                    .processes()
                    .iter()
                    .filter(|(_, process)| process.exe() == Some(herdr))
                    .map(|(pid, _)| *pid),
            );
        }
        let fogde = self.usage(&children, &roots);

        Sample {
            at: now(),
            cpu: fogde.cpu,
            memory: Memory {
                used: fogde.memory,
                total: self.system.total_memory(),
            },
            disk: repos.and_then(|repos| self.disk(repos)),
            battery: battery(),
            terminals: shells
                .iter()
                .filter(|shell| self.system.process(Pid::from_u32(shell.pid)).is_some())
                .map(|shell| {
                    let usage = self.usage(&children, &[Pid::from_u32(shell.pid)]);

                    Load {
                        id: shell.terminal.clone(),
                        cpu: usage.cpu,
                        memory: usage.memory,
                    }
                })
                .collect(),
        }
    }

    fn disk(&self, repos: &Path) -> Option<Disk> {
        self.disks
            .list()
            .iter()
            .filter(|disk| repos.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().as_os_str().len())
            .map(|disk| Disk {
                free: disk.available_space(),
                total: disk.total_space(),
            })
    }

    fn children(&self) -> HashMap<Pid, Vec<Pid>> {
        let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
        for (pid, process) in self.system.processes() {
            if process.thread_kind().is_some() {
                continue;
            }

            if let Some(parent) = process.parent() {
                children.entry(parent).or_default().push(*pid);
            }
        }

        children
    }

    fn usage(&self, children: &HashMap<Pid, Vec<Pid>>, roots: &[Pid]) -> Usage {
        let cores = u32::try_from(self.system.cpus().len().max(1)).unwrap_or(u32::MAX);
        let mut cpu = 0.0;
        let mut memory = 0;
        let mut seen = HashSet::new();
        let mut pending = roots.to_vec();
        while let Some(pid) = pending.pop() {
            if !seen.insert(pid) {
                continue;
            }

            if let Some(process) = self.system.process(pid) {
                cpu += f64::from(process.cpu_usage());
                memory += process.memory();
            }
            pending.extend(children.get(&pid).into_iter().flatten());
        }

        Usage {
            cpu: cpu / f64::from(cores),
            memory,
        }
    }
}

fn processes() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_exe(UpdateKind::OnlyIfNotSet)
}

fn battery() -> Option<Battery> {
    let manager = Manager::new().ok()?;
    let mut discharging = false;
    let mut watts = 0.0;
    let mut full = 0.0;
    for battery in manager.batteries().ok()?.flatten() {
        discharging |= battery.state() == State::Discharging;
        watts += f64::from(battery.energy_rate().get::<watt>().abs());
        full += f64::from(battery.energy_full().get::<watt_hour>());
    }

    (discharging && watts > 0.0 && full > 0.0).then(|| Battery {
        watts,
        drain: watts / full * 100.0,
    })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or_default()
}
