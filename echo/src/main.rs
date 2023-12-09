/// ```bash
/// $ cargo build
/// $ maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10 --log-stderr
/// ````
use async_trait::async_trait;
use maelstrom::protocol::{Message, MessageBody};
use maelstrom::{done, Node, Result, Runtime};
use serde_json::{json, Map};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub(crate) fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::new());
    Runtime::new().with_handler(handler).run().await
}

#[derive(Clone, Default)]
struct Handler {
    message_ids: Arc<Mutex<HashSet<u64>>>,
}

impl Handler {
    fn new() -> Handler {
        return Handler {
            message_ids: Arc::new(Mutex::new(HashSet::new())),
        };
    }

    fn store_message(&self, message_id: u64) {
        Arc::clone(&self.message_ids)
            .lock()
            .unwrap()
            .insert(message_id);
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
        if req.get_type() == "echo" {
            let reply = req.body.clone().with_type("echo_ok");
            return runtime.reply(req, reply).await;
        }
        if req.get_type() == "generate" {
            let reply = req.body.clone().with_type("generate_ok");
            let mut extra = Map::new();
            extra.insert("id".to_string(), json!(Uuid::new_v4().to_string()));
            return runtime.reply(req, MessageBody { extra, ..reply }).await;
        }
        if req.get_type() == "broadcast" {
            let reply = req.body.clone().with_type("broadcast_ok");
            self.store_message(reply.extra.get("message").unwrap().as_u64().unwrap());
            return runtime
                .reply(
                    req,
                    MessageBody {
                        extra: Map::new(),
                        ..reply
                    },
                )
                .await;
        }
        if req.get_type() == "read" {
            let reply = req.body.clone().with_type("read_ok");
            let mut extra = Map::new();
            extra.insert("messages".to_string(), self.retreieve_messages()?.into());
            return runtime.reply(req, MessageBody { extra, ..reply }).await;
        }
        if req.get_type() == "topology" {
            let reply = req.body.clone().with_type("topology_ok");
            return runtime
                .reply(
                    req,
                    MessageBody {
                        extra: Map::new(),
                        ..reply
                    },
                )
                .await;
        }

        done(runtime, req)
    }
}
