use anyhow::Context;
use futures::{
    stream::{StreamExt, TryStreamExt},
    FutureExt,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

// To start server, we need to know some free port.
// Even if there is a way to get this information, it would
// suffer from race conditions.
// That's why, we simply select random port and try using it.
// 20 iterations give negligible probality of failure.
const BIND_ATTEMPTS: usize = 20;

pub struct Client {
    endpoint: String,
    transport: reqwest::Client,
}

impl Client {
    pub async fn start<T: Serialize>(
        &self,
        url: &str,
        input: &T,
    ) -> anyhow::Result<pps_api::OperationInfo> {
        if !url.starts_with('/') {
            anyhow::bail!("url does not start with /")
        }
        let res = self
            .transport
            .post(format!("{}{}", self.endpoint, url))
            .json(input)
            .send()
            .await?;
        res.error_for_status_ref()?;
        let resp = res.json().await?;
        Ok(resp)
    }

    pub async fn get_operation(&self, op_id: Uuid) -> anyhow::Result<pps_api::Operation> {
        let url = format!("{}/operations/{}", self.endpoint, op_id.to_hyphenated());
        let op = self
            .transport
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(op)
    }

    pub fn operation_events_stream<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        op_id: Uuid,
    ) -> futures::stream::BoxStream<'static, anyhow::Result<T>> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let url = format!("{}/operations/{}", self.endpoint, op_id.to_hyphenated());
        let tr = self.transport.clone();
        let mut had_error = false;
        let closed_fut = {
            let tx = tx.clone();
           async move { tx.closed().await;}
        };
        let event_stream = futures::stream::iter(0..)
            .then(move |skip: usize| {
                tr.get(url.clone())
                    .query(&[("skip_events", skip)])
                    .query(&[("wait", true)])
                    .send()
            })
            .and_then(|resp| async move { resp.error_for_status()?.json().await })
            .map_ok(|op: pps_api::Operation| op.events.get(0).cloned())
            .map_err(anyhow::Error::new)
            .filter_map(|it| async { it.transpose() })
            .and_then(|val| async move {
                serde_json::from_value(val).context("failed to deserialize event")
            })
            .take_while(move |res| {
                let take = if had_error {
                    false
                } else {
                    had_error = res.is_err();
                    true
                };
                async move { take }
            })
            .take_until(closed_fut);
        tokio::task::spawn(async move {
            event_stream.for_each(|item| tx.send(item).map(drop)).await;
        });
        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
}

#[tracing::instrument(skip(cancel))]
pub(crate) async fn create_server(
    cancel: tokio_util::sync::CancellationToken,
) -> anyhow::Result<Client> {
    // TODO provide way to customize port or port range
    tracing::info!("launching embedded server");
    let mut last_error = None;
    for _ in 0..BIND_ATTEMPTS {
        let port: u16 = rand::random();
        let (startup_tx, startup_rx) = tokio::sync::oneshot::channel();
        tokio::task::spawn(pps_server::serve(port, cancel.clone(), startup_tx));
        match startup_rx.await.context("server task did not startup")? {
            Ok(_) => {
                let endpoint = format!("http://127.0.0.1:{}", port);
                let client = Client {
                    endpoint,
                    transport: reqwest::Client::new(),
                };
                return Ok(client);
            }
            Err(err) => {
                tracing::warn!(error=?err, port = port, "bind attempt unsuccessful");
                last_error = Some(err)
            }
        }
    }
    Err(last_error.expect("BIND_ATTEMPTS != 0"))
}
