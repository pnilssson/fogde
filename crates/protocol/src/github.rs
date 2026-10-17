use serde::Serialize;
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct SignInCode {
    pub code: String,
    pub url: String,
}

pub struct GetUser;

impl Method for GetUser {
    const NAME: &'static str = "github.user";
    type Params = ();
    type Result = Option<User>;
}

pub struct StartSignIn;

impl Method for StartSignIn {
    const NAME: &'static str = "github.startSignIn";
    type Params = ();
    type Result = SignInCode;
}

pub struct FinishSignIn;

impl Method for FinishSignIn {
    const NAME: &'static str = "github.finishSignIn";
    type Params = ();
    type Result = User;
}
