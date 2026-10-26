use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::machines::MachineId;

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct Sample {
    #[ts(type = "number")]
    pub at: i64,
    pub cpu: f64,
    pub memory: Memory,
    pub disk: Option<Disk>,
    pub battery: Option<Battery>,
    pub terminals: Vec<Load>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Memory {
    #[ts(type = "number")]
    pub used: u64,
    #[ts(type = "number")]
    pub total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Disk {
    #[ts(type = "number")]
    pub free: u64,
    #[ts(type = "number")]
    pub total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
pub struct Battery {
    pub watts: f64,
    pub drain: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct Load {
    pub id: String,
    pub cpu: f64,
    #[ts(type = "number")]
    pub memory: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Space {
    pub path: String,
    #[ts(type = "number")]
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, TS)]
pub struct SamplesParams {
    #[ts(type = "number")]
    pub since: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Metric {
    Cpu,
    Memory,
    Battery,
    Latency,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Bar {
    pub machine: MachineId,
    pub metrics: Vec<Metric>,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            machine: MachineId::Computer,
            metrics: vec![
                Metric::Cpu,
                Metric::Memory,
                Metric::Battery,
                Metric::Latency,
            ],
        }
    }
}

pub struct ListSamples;

impl Method for ListSamples {
    const NAME: &'static str = "monitor.samples";
    type Params = SamplesParams;
    type Result = Vec<Sample>;
}

pub struct ListSpaces;

impl Method for ListSpaces {
    const NAME: &'static str = "monitor.spaces";
    type Params = Vec<String>;
    type Result = Vec<Space>;
}

pub struct GetBar;

impl Method for GetBar {
    const NAME: &'static str = "monitor.getBar";
    type Params = ();
    type Result = Bar;
}

pub struct SetBar;

impl Method for SetBar {
    const NAME: &'static str = "monitor.setBar";
    type Params = Bar;
    type Result = ();
}
