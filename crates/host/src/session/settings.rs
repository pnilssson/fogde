use protocol::onboarding::{FinishOnboarding, GetOnboarded};
use protocol::rpc::Failure;
use protocol::theme::{GetTheme, SetTheme};
use serde_json::Value;

use super::{Session, parse, reply};

impl Session {
    pub(super) async fn onboarded(&self, params: Value) -> Result<Value, Failure> {
        parse::<GetOnboarded>(params)?;

        reply::<GetOnboarded, _>(self.shared.settings.onboarded().await)
    }

    pub(super) async fn finish_onboarding(&self, params: Value) -> Result<Value, Failure> {
        parse::<FinishOnboarding>(params)?;

        reply::<FinishOnboarding, _>(self.shared.settings.finish_onboarding().await)
    }

    pub(super) async fn theme(&self, params: Value) -> Result<Value, Failure> {
        parse::<GetTheme>(params)?;

        reply::<GetTheme, _>(self.shared.settings.theme().await)
    }

    pub(super) async fn set_theme(&self, params: Value) -> Result<Value, Failure> {
        let theme = parse::<SetTheme>(params)?;

        reply::<SetTheme, _>(self.shared.settings.set_theme(theme).await)
    }
}
