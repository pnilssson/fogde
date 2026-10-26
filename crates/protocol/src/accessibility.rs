use serde::{Deserialize, Serialize};

use crate::Method;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility {
    pub screen_reader: bool,
}

pub struct GetAccessibility;

impl Method for GetAccessibility {
    const NAME: &'static str = "accessibility.get";
    type Params = ();
    type Result = Accessibility;
}

pub struct SetAccessibility;

impl Method for SetAccessibility {
    const NAME: &'static str = "accessibility.set";
    type Params = Accessibility;
    type Result = ();
}
