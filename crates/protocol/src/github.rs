use serde::{Deserialize, Serialize};

use crate::Method;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub host: String,
    pub login: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

pub struct CancelSignIn;

impl Method for CancelSignIn {
    const NAME: &'static str = "github.cancelSignIn";
    type Params = ();
    type Result = ();
}
