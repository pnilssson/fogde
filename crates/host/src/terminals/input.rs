use std::future;

use protocol::rpc::Failure;
use protocol::terminals::{
    PasteImage, PasteImageParams, ResizeTerminal, ResizeTerminalParams, TerminalInput,
    TerminalInputParams,
};

use super::images;
use super::stream::Control;
use crate::rpc::{Handler, failure};
use crate::session::Session;

impl Handler for TerminalInput {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalInputParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Input(params.data.into_bytes())),
        )
    }
}

impl Handler for ResizeTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: ResizeTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Resize(params.size)),
        )
    }
}

impl Handler for PasteImage {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: PasteImageParams) -> Result<(), Failure> {
        let path = images::save(&session.shared.images(), params.image, params.data)
            .await
            .map_err(failure)?;
        let paste = images::paste(&path);

        session
            .terminals
            .send(&params.id, Control::Input(paste.into_bytes()))
    }
}
