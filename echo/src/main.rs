use async_trait::async_trait;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub(crate) fn main() -> Result<()> {
    // let data = r#"{"echo":"Please echo 13","type":"echo","msg_id":1}"#;
    // let body: RequestBody = serde_json::from_str(data)?;
    // println!("{:?}", body);
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::new());
    Runtime::new().with_handler(handler).run().await
}

#[derive(Clone, Default)]
struct Handler {
    message_ids: Arc<Mutex<HashSet<u64>>>,
    neighbors: Arc<Mutex<Vec<String>>>,
}

impl Handler {
    fn new() -> Handler {
        return Handler {
            message_ids: Arc::new(Mutex::new(HashSet::new())),
            neighbors: Arc::new(Mutex::new(Vec::new())),
        };
    }

    fn store_message(&self, message_id: u64) {
        Arc::clone(&self.message_ids)
            .lock()
            .unwrap()
            .insert(message_id);
    }

    fn set_neighbors(&self, node_ids: &mut Vec<String>) {
        Arc::clone(&self.neighbors).lock().unwrap().append(node_ids);
    }

    fn retreieve_messages(&self) -> Result<Vec<u64>> {
        Ok(Arc::clone(&self.message_ids)
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .collect())
    }
}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        eprintln!("{:?}", req.body);
        let body: RequestBody = req.body.as_obj()?;
        let msg_id = req.body.msg_id;
        match body {
            RequestBody::Init {} => done(runtime, req),
            RequestBody::Echo { echo } => {
                runtime
                    .reply(req, ResponseBody::EchoOk { echo, msg_id })
                    .await
            }
            RequestBody::Generate {} => {
                runtime
                    .reply(
                        req,
                        ResponseBody::GenerateOk {
                            id: Uuid::new_v4().to_string(),
                        },
                    )
                    .await
            }
            RequestBody::Broadcast { message } => {
                self.store_message(message);
                runtime.reply_ok(req).await
            }
            RequestBody::Topology { topology } => {
                let mut neighbors = topology.get(runtime.node_id()).unwrap().clone();
                self.set_neighbors(&mut neighbors);
                runtime.reply_ok(req).await
            }
            RequestBody::Read {} => {
                runtime
                    .reply(
                        req,
                        ResponseBody::ReadOk {
                            messages: self.retreieve_messages()?,
                        },
                    )
                    .await
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
enum RequestBody {
    Init {},
    Echo {
        echo: String,
    },
    Generate {},
    Broadcast {
        message: u64,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    Read {},
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum ResponseBody {
    EchoOk { echo: String, msg_id: u64 },
    GenerateOk { id: String },
    ReadOk { messages: Vec<u64> },
}
