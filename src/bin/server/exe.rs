use std::{convert::TryFrom, net::SocketAddr};

use futures::{channel::mpsc, executor::block_on};
use kube::Api;
use log::info;
use tokio::{
    select,
    signal::{ctrl_c, unix::SignalKind},
    task,
};
use tonic::transport::Server;

use crate::{console::Console, controller::ControlService, judge_server::JudgeServer};
use dualoj_judge::proto::{
    controller_server::ControllerServer, judger::judger_server::JudgerServer,
};

pub struct Executor {
    controller: ControllerServer<ControlService>,
    judger: JudgerServer<JudgeServer>,
    controller_addr: SocketAddr,
    judger_addr: SocketAddr,
}

impl Executor {
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Server listen on {}", self.controller_addr);

        let controller_thread = task::spawn(
            Server::builder()
                .add_service(self.controller)
                .serve(self.controller_addr),
        );
        let judger_thread = task::spawn(
            Server::builder()
                .add_service(self.judger)
                .serve(self.judger_addr),
        );
        let mut term = tokio::signal::unix::signal(SignalKind::terminate())?;

        select! {
            _ = ctrl_c() => {}
            _ = term.recv() => {}
        }

        controller_thread.abort();
        judger_thread.abort();
        Ok(())
    }
}

impl TryFrom<Console> for Executor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: Console) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::channel(5); // TODO!: tuning this buffer size
        let client = block_on(kube::Client::try_default())?;
        let judger_addr = SocketAddr::new(value.pod_env.ip, value.judger_port);
        let controller_addr = SocketAddr::new(value.pod_env.ip, value.controller_port);

        // TODO!: customized target namespace.
        let job_api = Api::namespaced(client.clone(), &value.pod_env.namespace);
        let pod_api = Api::namespaced(client, &value.pod_env.namespace);

        let controller = ControllerServer::new(ControlService {
            archive_size_limit: value.archive_size_limit,
            buildkit: value.buildkit,
            registry: value.registry,
            pod_env: value.pod_env,
            job_poster: tx,
            judger_addr,

            job_api,
            pod_api,
        });
        let judger = JudgerServer::new(JudgeServer::new(rx));

        Ok(Executor {
            controller,
            judger,
            controller_addr,
            judger_addr,
        })
    }
}
