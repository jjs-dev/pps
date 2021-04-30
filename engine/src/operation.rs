//! Interface between REST api and actual logic

use tokio::sync::mpsc;

/// Represents running pps-engine API operation.
/// Each operation receives a series of events.
///
/// Update is a single event for the operation.
pub struct Operation<Update> {
    rx: mpsc::Receiver<ChannelMessage<Update>>,
    finish: Option<Outcome>,
}

impl<Update> Operation<Update> {
    /// Receives next update. When returns None, you can use `finish` method.
    pub async fn next_update(&mut self) -> Option<Update> {
        if self.finish.is_some() {
            panic!("next_update called after receiving None");
        }
        let msg = self.rx.recv().await.unwrap_or_else(|| {
            ChannelMessage::Done(Outcome::Error(anyhow::Error::msg(
                "operation was stopped in unexpected way",
            )))
        });
        match msg {
            ChannelMessage::Done(d) => {
                self.finish = Some(d);
                None
            }
            ChannelMessage::Progress(p) => Some(p),
        }
    }

    /// Returns operation outcome
    pub fn outcome(self) -> Outcome {
        self.finish
            .expect("outcome called before receiving None from next_update")
    }
}

enum ChannelMessage<Update> {
    Progress(Update),
    Done(Outcome),
}

/// Outcome of the operation
pub enum Outcome {
    /// Operation has finished successfully.
    Finish,
    /// Operation failed, error is attached.
    Error(anyhow::Error),
    /// Operation was cancelled successfully.
    Cancelled,
}

/// Used to report progress on operation
pub(crate) struct ProgressWriter<Update> {
    tx: mpsc::Sender<ChannelMessage<Update>>,
}

impl<Update> ProgressWriter<Update> {
    /// Publishes an update
    pub async fn send(&mut self, ev: Update) {
        self.tx.send(ChannelMessage::Progress(ev)).await.ok();
    }

    pub async fn finish(self, res: anyhow::Result<()>) {
        let out = match res {
            Ok(_) => Outcome::Finish,
            Err(err) => Outcome::Error(err),
        };
        self.tx.send(ChannelMessage::Done(out)).await.ok();
    }
}

pub(crate) fn start<U>() -> (Operation<U>, ProgressWriter<U>) {
    let (tx, rx) = mpsc::channel(1);

    let op = Operation { rx, finish: None };
    let pw = ProgressWriter { tx };

    (op, pw)
}
