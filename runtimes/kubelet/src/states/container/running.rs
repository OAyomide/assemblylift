use crossbeam_channel::Receiver;
use kubelet::container::state::prelude::*;
use tracing::{debug, instrument, warn};

use crate::ProviderState;

use super::terminated::Terminated;
use super::ContainerState;

/// The container is starting.
#[derive(Debug, TransitionTo)]
#[transition_to(Terminated)]
pub struct Running {
    rx: Receiver<Status>,
}

impl Running {
    pub fn new(rx: Receiver<Status>) -> Self {
        Running { rx }
    }
}

#[async_trait::async_trait]
impl State<ContainerState> for Running {
    #[instrument(level = "info", skip(self, _shared_state, _state, _container))]
    async fn next(
        mut self: Box<Self>,
        _shared_state: SharedState<ProviderState>,
        _state: &mut ContainerState,
        _container: Manifest<Container>,
    ) -> Transition<ContainerState> {
        debug!("Awaiting container status updates");
        while let Ok(status) = self.rx.recv() {
            debug!(?status, "Got status update from WASI Runtime");
            if let Status::Terminated {
                failed, message, ..
            } = status
            {
                return Transition::next(self, Terminated::new(message, failed));
            }
        }
        warn!("WASI Runtime channel hung up");
        Transition::next(
            self,
            Terminated::new("WASI Runtime channel hung up".to_string(), true),
        )
    }

    async fn status(
        &self,
        _state: &mut ContainerState,
        _container: &Container,
    ) -> anyhow::Result<Status> {
        Ok(Status::running())
    }
}
