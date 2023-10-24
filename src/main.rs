use std::net::SocketAddr;
use shuttle_runtime::{Error, Service};
use tokio::{select, task};

mod reply;
mod command;
mod steam;
mod config;
mod rule;

pub struct KaixuHelperService;

#[shuttle_runtime::async_trait]
impl Service for KaixuHelperService {
    async fn bind(mut self, _addr: SocketAddr) -> Result<(), Error> {
        select! {
            _ = start() => {},
        }
        Ok(())
    }
}

#[shuttle_runtime::main]
async fn init() -> Result<KaixuHelperService, Error> {
    Ok(KaixuHelperService)
}


async fn start() {
    let reply = task::spawn(reply::start());
    let command = task::spawn(command::start());
    let steam = task::spawn(steam::start());

    reply.await.unwrap();
    command.await.unwrap();
    steam.await.unwrap();
}