use std::future;

use protocol::rpc::Failure;
use protocol::terminals::{
    ClickTerminal, ClickTerminalParams, PasteImage, PasteImageParams, ResizeTerminal,
    ScrollTerminal, ScrollTerminalParams, TerminalInput, TerminalInputParams, TerminalKey,
    TerminalKeyParams, WatchTerminalParams,
};

use super::stream::Control;
use super::{herdr, images};
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

impl Handler for TerminalKey {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalKeyParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Key(herdr::key(params.key))),
        )
    }
}

impl Handler for ResizeTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: WatchTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Resize(params.size)),
        )
    }
}

impl Handler for ScrollTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: ScrollTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let id = params.id.clone();

        future::ready(session.terminals.send(&id, Control::Scroll(params)))
    }
}

impl Handler for ClickTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: ClickTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let id = params.id.clone();

        future::ready(session.terminals.send(&id, Control::Click(params)))
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
