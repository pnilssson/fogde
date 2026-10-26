use serde::{Deserialize, Serialize};

use crate::paths;

pub const START: &str = "start";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Selection {
    pub machine: String,
    pub id: String,
}

impl Selection {
    pub fn new(machine: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            machine: machine.into(),
            id: id.into(),
        }
    }

    pub fn tile(&self) -> String {
        format!("{}:{}", self.machine, self.id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct View {
    pub machine: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout: Option<String>,
}

impl View {
    pub fn workspace(machine: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            machine: machine.into(),
            id: id.into(),
            checkout: None,
        }
    }

    pub fn key(&self) -> String {
        match &self.checkout {
            Some(checkout) => format!("{}/{}/{checkout}", self.machine, self.id),
            None => format!("{}/{}", self.machine, self.id),
        }
    }

    pub fn home(&self) -> View {
        View::workspace(self.machine.clone(), self.id.clone())
    }

    pub fn shows(&self, checkout: Option<&str>) -> bool {
        self.checkout.as_deref() == checkout
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Tile {
    Terminal {
        machine: String,
        id: String,
    },
    Start {
        folder: String,
    },
    Notes {
        key: String,
        machine: String,
        note: Option<String>,
    },
}

impl Tile {
    pub fn terminal(selection: &Selection) -> Self {
        Self::Terminal {
            machine: selection.machine.clone(),
            id: selection.id.clone(),
        }
    }

    pub fn id(&self) -> String {
        match self {
            Self::Terminal { machine, id } => format!("{machine}:{id}"),
            Self::Start { .. } => START.to_owned(),
            Self::Notes { key, .. } => key.clone(),
        }
    }

    pub fn selection(&self) -> Option<Selection> {
        match self {
            Self::Terminal { machine, id } => Some(Selection::new(machine.clone(), id.clone())),
            _ => None,
        }
    }
}

pub fn note_title(note: Option<&str>) -> String {
    match note {
        Some(note) => {
            let name = paths::name(note);
            if name.to_ascii_lowercase().ends_with(".md") {
                name[..name.len() - 3].to_owned()
            } else {
                name.to_owned()
            }
        }
        None => "Notes".to_owned(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dragged {
    Terminal(Selection),
    Tile { id: String, view: Option<View> },
    Note { machine: String, path: String },
}

impl Dragged {
    pub fn tile_id(&self) -> Option<String> {
        match self {
            Self::Terminal(selection) => Some(selection.tile()),
            Self::Tile { id, .. } => Some(id.clone()),
            Self::Note { .. } => None,
        }
    }
}
