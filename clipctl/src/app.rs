use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::ipc::{Client, Request, RequestKind};
use crate::ui::{HandleOutcome, TerminalUi, UiEvent};

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        tokio::spawn({
            let event_tx = event_tx.clone();
            async move {
                loop {
                    let read_result =
                        tokio::task::spawn_blocking(crossterm::event::read).await;
                    match read_result {
                        Ok(Ok(event)) => {
                            if event_tx.send(UiEvent::Input(event)).is_err() {
                                break;
                            }
                        }
                        Ok(Err(err)) => {
                            tracing::error!(%err, "failed to read crossterm event");
                            break;
                        }
                        Err(join_err) => {
                            tracing::error!(%join_err, "event reader task panicked");
                            break;
                        }
                    }
                }
            }
        });

        let mut ui = TerminalUi::new()?;
        let mut client = Client::connect().await?;

        client
            .send(&Request {
                kind: RequestKind::List,
            })
            .await
            .context("failed to request initial history")?;

        // Wait for the initial response before starting the UI
        let initial_response = client.next_message().await?;
        ui.ingest_response(initial_response)?;

        let mut tick = time::interval(Duration::from_millis(75));
        ui.draw()?;

        loop {
            tokio::select! {
                _ = tick.tick() => {
                    ui.draw()?;
                }
                event = event_rx.recv() => match event {
                    Some(event) => {
                        let HandleOutcome { should_exit, request } = ui.handle_event(event)?;
                        if let Some(req) = request {
                            client.send(&req).await?;
                        }
                        if should_exit {
                            break;
                        }
                    }
                    None => break,
                },
                response = client.next_message() => {
                    ui.ingest_response(response?)?;
                    ui.draw()?;  // Immediately redraw after receiving new data
                }
            }
        }

        Ok(())
    }
}

