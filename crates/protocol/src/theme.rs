use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

pub struct GetTheme;

impl Method for GetTheme {
    const NAME: &'static str = "theme.get";
    type Params = ();
    type Result = Theme;
}

pub struct SetTheme;

impl Method for SetTheme {
    const NAME: &'static str = "theme.set";
    type Params = Theme;
    type Result = ();
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Material {
    #[default]
    Translucent,
    Solid,
}

pub struct GetMaterial;

impl Method for GetMaterial {
    const NAME: &'static str = "theme.getMaterial";
    type Params = ();
    type Result = Material;
}

pub struct SetMaterial;

impl Method for SetMaterial {
    const NAME: &'static str = "theme.setMaterial";
    type Params = Material;
    type Result = ();
}
