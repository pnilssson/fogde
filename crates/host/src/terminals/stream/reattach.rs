use std::sync::Arc;

use herdr_client::{Error, Herdr, Stream};
use protocol::terminals::{Display, Size};

use super::facts;

pub trait Reattach: Send + Sync + 'static {
    fn attach(
        &self,
        id: &str,
        size: Size,
        display: Display,
    ) -> impl Future<Output = Result<Stream, Error>> + Send;
}

impl Reattach for Arc<Herdr> {
    async fn attach(&self, id: &str, size: Size, display: Display) -> Result<Stream, Error> {
        self.client()
            .await?
            .attach(id, size.cols, size.rows, facts(display))
            .await
    }
}

pub(super) fn restarting(error: &Error) -> bool {
    matches!(error, Error::Herdr { kind, .. } if kind == "unavailable")
}

pub(super) fn waiting(error: &Error) -> bool {
    restarting(error)
        || matches!(
            error,
            Error::Closed | Error::Io(_) | Error::Timeout | Error::Start(_)
        )
}
