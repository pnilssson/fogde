use crate::Method;

pub struct GetOnboarded;

impl Method for GetOnboarded {
    const NAME: &'static str = "onboarding.done";
    type Params = ();
    type Result = bool;
}

pub struct FinishOnboarding;

impl Method for FinishOnboarding {
    const NAME: &'static str = "onboarding.finish";
    type Params = ();
    type Result = ();
}
