/// ```bash
/// $ cargo build
/// $ maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10 --log-stderr
/// ````
use async_trait::async_trait;
use maelstrom::protocol::{Message, MessageBody};
use maelstrom::{done, Node, Result, Runtime};
use serde_json::{json, Map};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::default());
    Runtime::new().with_handler(handler).run().await
}

#[derive(Clone, Default)]
struct Handler {}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        if req.get_type() == "echo" {
            let echo = req.body.clone().with_type("echo_ok");
            return runtime.reply(req, echo).await;
        }
        if req.get_type() == "generate" {
            let reply = req.body.clone().with_type("generate_ok");
            let mut extra = Map::new();
            extra.insert("id".to_string(), json!(Uuid::new_v4().to_string()));
            return runtime.reply(req, MessageBody { extra, ..reply }).await;
        }

        done(runtime, req)
    }
}
