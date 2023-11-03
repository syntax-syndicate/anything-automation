use std::path::PathBuf;

use anything_graph::{Flow, Flowfile};
use anything_persistence::{FlowRepo, FlowRepoImpl};
use anything_store::{types::ChangeMessage, FileStore};
use ractor::{async_trait, message, Actor, ActorProcessingErr, ActorRef};

use crate::{CoordinatorActorResult, CoordinatorError, CoordinatorResult};

#[derive(Debug, Clone)]
pub enum SystemMessage {
    StoreChanged(ChangeMessage),
}

impl SystemMessage {
    fn next(&self) -> Self {
        match self {
            SystemMessage::StoreChanged(_) => SystemMessage::StoreChanged(ChangeMessage::default()),
        }
    }

    fn print(&self) {
        match self {
            SystemMessage::StoreChanged(_) => println!("Store changed"),
        }
    }
}

pub struct SystemActor;
pub struct SystemActorState {
    pub file_store: FileStore,
    pub flow_repo: FlowRepoImpl,
}

#[async_trait]
impl Actor for SystemActor {
    type Msg = SystemMessage;
    type State = SystemActorState;
    type Arguments = SystemActorState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(args)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> CoordinatorActorResult<()> {
        match message {
            SystemMessage::StoreChanged(ChangeMessage {
                change_type: anything_store::types::SystemChangeType::Flows,
                ..
            }) => {
                tracing::debug!("Reloading flows");
                self.reload_flows(myself, message, state).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

// // impl Actor for SystemActor {
// //     type Context = Context<Self>;

// //     fn on_start(&mut self, ctx: &mut Context<Self>) {
// //         println!("in started: {:?}", System::current());
// //         // ctx.run_later(Duration::from_millis(200), |act, ctx| {
// //         //     println!("started later ");
// //         // });
// //     }
// // }

// // impl Handler<SystemMessages> for SystemActor {
// //     type Result = crate::error::CoordinatorResult<()>;

// //     fn handle(&mut self, msg: SystemMessages, _ctx: &mut Context<Self>) -> Self::Result {
// //         println!("Handling system message got a message: {:?}", msg);
// //         match msg {
// //             SystemMessages::StoreChanged(_) => {
// //                 println!("Store changed");
// //                 Ok(())
// //             }
// //         }
// //     }
// // }

impl SystemActor {
    pub async fn reload_flows(
        &self,
        _myself: ActorRef<<SystemActor as Actor>::Msg>,
        _message: <SystemActor as Actor>::Msg,
        state: &mut <SystemActor as Actor>::State,
    ) -> CoordinatorResult<()> {
        let file_store = state.file_store.clone();
        let flow_repo = state.flow_repo.clone();
        let root_dir = file_store.store_path(&["flows"]);

        let flow_repo = flow_repo.clone();

        let flow_files: Vec<PathBuf> = anything_common::utils::anythingfs::read_flow_directories(
            root_dir,
            vec!["toml".to_string()],
        )
        .map_err(|e| {
            tracing::error!("error when reading flow directories: {:#?}", e);
            CoordinatorError::AnythingError(e)
        })?;

        for flow_file_path in flow_files {
            let flow = match Flowfile::from_file(flow_file_path) {
                Ok(flow) => flow,
                Err(e) => {
                    tracing::error!("error when parsing flow file: {:#?}", e);
                    continue;
                }
            };
            let flow: Flow = flow.into();
            match flow_repo.create_or_update_flow(flow.into()).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("error when saving flow: {:#?}", e);
                    continue;
                }
            };
        }

        Ok(())
    }
}
