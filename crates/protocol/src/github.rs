use serde::Serialize;
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Serialize, TS)]
pub struct Account {
    pub host: String,
    pub login: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct SignInCode {
    pub code: String,
    pub url: String,
}

pub struct GetAccounts;

impl Method for GetAccounts {
    const NAME: &'static str = "github.accounts";
    type Params = ();
    type Result = Vec<Account>;
}

pub struct StartSignIn;

impl Method for StartSignIn {
    const NAME: &'static str = "github.startSignIn";
    type Params = String;
    type Result = SignInCode;
}

pub struct FinishSignIn;

impl Method for FinishSignIn {
    const NAME: &'static str = "github.finishSignIn";
    type Params = ();
    type Result = ();
}
