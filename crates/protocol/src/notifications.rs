use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Notifications {
    pub enabled: bool,
    pub sound: bool,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: true,
        }
    }
}

pub struct GetNotifications;

impl Method for GetNotifications {
    const NAME: &'static str = "notifications.get";
    type Params = ();
    type Result = Notifications;
}

pub struct SetNotifications;

impl Method for SetNotifications {
    const NAME: &'static str = "notifications.set";
    type Params = Notifications;
    type Result = ();
}
