use client::Client;
use protocol::terminals::{
    Image, Key, PasteImage, PasteImageParams, ResizeTerminal, ScrollTerminal, ScrollTerminalParams,
    SeekTerminal, SeekTerminalParams, Size, TerminalInput, TerminalInputParams, TerminalKey,
    TerminalKeyParams, WatchTerminalParams,
};
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq)]
pub enum Outgoing {
    Data(String),
    Key(Key),
    Image(Image, String),
    Scroll(ScrollTerminalParams),
    Seek(u32),
    Resize(Size),
}

pub struct Failure {
    pub title: &'static str,
    pub error: client::Error,
}

pub fn coalesce(batch: Vec<Outgoing>) -> Vec<Outgoing> {
    let mut merged: Vec<Outgoing> = Vec::with_capacity(batch.len());
    for item in batch {
        match (merged.last_mut(), item) {
            (Some(Outgoing::Data(text)), Outgoing::Data(more)) => text.push_str(&more),
            (_, Outgoing::Seek(offset)) => {
                merged.retain(|known| !matches!(known, Outgoing::Seek(_)));
                merged.push(Outgoing::Seek(offset));
            }
            (_, Outgoing::Resize(size)) => {
                merged.retain(|known| !matches!(known, Outgoing::Resize(_)));
                merged.push(Outgoing::Resize(size));
            }
            (_, item) => merged.push(item),
        }
    }
    merged
}

pub async fn run(
    client: Client,
    id: String,
    mut outgoing: mpsc::UnboundedReceiver<Outgoing>,
    failures: mpsc::Sender<Failure>,
) {
    while let Some(first) = outgoing.recv().await {
        let mut batch = vec![first];
        while let Ok(next) = outgoing.try_recv() {
            batch.push(next);
        }

        for item in coalesce(batch) {
            let id = id.clone();
            let (title, result) = match item {
                Outgoing::Data(data) => (
                    "Could not send your input",
                    client
                        .call::<TerminalInput>(TerminalInputParams { id, data })
                        .await,
                ),
                Outgoing::Key(key) => (
                    "Could not send your input",
                    client
                        .call::<TerminalKey>(TerminalKeyParams { id, key })
                        .await,
                ),
                Outgoing::Image(image, data) => (
                    "Could not paste",
                    client
                        .call::<PasteImage>(PasteImageParams { id, image, data })
                        .await,
                ),
                Outgoing::Scroll(params) => (
                    "Could not scroll the terminal",
                    client
                        .call::<ScrollTerminal>(ScrollTerminalParams { id, ..params })
                        .await,
                ),
                Outgoing::Seek(offset) => (
                    "Could not scroll the terminal",
                    client
                        .call::<SeekTerminal>(SeekTerminalParams { id, offset })
                        .await,
                ),
                Outgoing::Resize(size) => (
                    "Could not resize the terminal",
                    client
                        .call::<ResizeTerminal>(WatchTerminalParams { id, size })
                        .await,
                ),
            };
            if let Err(error) = result
                && failures.send(Failure { title, error }).await.is_err()
            {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_merges_and_only_the_last_seek_and_size_go_out() {
        let batch = vec![
            Outgoing::Data("a".into()),
            Outgoing::Data("b".into()),
            Outgoing::Seek(3),
            Outgoing::Resize(Size { cols: 80, rows: 24 }),
            Outgoing::Seek(5),
            Outgoing::Key(Key::ShiftEnter),
            Outgoing::Data("c".into()),
            Outgoing::Resize(Size {
                cols: 100,
                rows: 30,
            }),
        ];

        assert_eq!(
            coalesce(batch),
            vec![
                Outgoing::Data("ab".into()),
                Outgoing::Seek(5),
                Outgoing::Key(Key::ShiftEnter),
                Outgoing::Data("c".into()),
                Outgoing::Resize(Size {
                    cols: 100,
                    rows: 30
                }),
            ]
        );
    }
}
