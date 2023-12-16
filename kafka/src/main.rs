use async_trait::async_trait;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::new());
    Runtime::new().with_handler(handler).run().await
}

struct CommitLog {
    committed_offset: Option<usize>,
    messages: Vec<u64>,
}

struct Handler {
    offsets: Arc<Mutex<HashMap<String, CommitLog>>>,
}

impl Handler {
    fn new() -> Handler {
        return Handler {
            offsets: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    fn add_message(&self, key: String, msg: u64) -> usize {
        let mut map = self.offsets.lock().unwrap();
        let commit_log = map.entry(key).or_insert(CommitLog {
            committed_offset: None,
            messages: vec![],
        });
        (*commit_log).messages.push(msg);
        (*commit_log).messages.len() - 1
    }

    fn get_messages(
        &self,
        key: &String,
        start_offset_inclusive: usize,
    ) -> Result<Vec<(usize, u64)>> {
        let map = self.offsets.lock().unwrap();
        let commit_log = map.get(key).ok_or("Could not find key")?;
        let mut result: Vec<(usize, u64)> = vec![];
        if start_offset_inclusive >= commit_log.messages.len() {
            return Err("Invalid offset".into());
        }
        for i in start_offset_inclusive..(commit_log.messages.len()) {
            result.push((
                i,
                *commit_log
                    .messages
                    .get(i)
                    .ok_or("Could not find message for offset")?,
            ));
        }
        Ok(result)
    }

    fn commit_offset(&self, key: String, offset: usize) -> Result<()> {
        let mut map = self.offsets.lock().unwrap();
        map.entry(key)
            .and_modify(|e| (*e).committed_offset = Some(offset));
        Ok(())
    }

    fn get_committed_offset(&self, key: &String) -> Result<usize> {
        let map = self.offsets.lock().unwrap();
        let commit_log = map.get(key).ok_or("Could not find key")?;
        Ok(commit_log.committed_offset.unwrap_or(0))
    }
}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        eprintln!("{:?}", req.body);
        let body: RequestBody = req.body.as_obj()?;
        // let msg_id = req.body.msg_id;
        match body {
            RequestBody::Init {} => done(runtime, req),
            RequestBody::Send { key, msg } => {
                runtime
                    .reply(
                        req,
                        ResponseBody::SendOk {
                            offset: __self.add_message(key, msg),
                        },
                    )
                    .await
            }
            RequestBody::Poll { offsets } => {
                runtime
                    .reply(
                        req,
                        ResponseBody::PollOk {
                            msgs: offsets
                                .iter()
                                .map(|(key, offset)| {
                                    (key.clone(), (self.get_messages(key, *offset).unwrap()))
                                })
                                .collect(),
                        },
                    )
                    .await
            }
            RequestBody::CommitOffsets { offsets } => {
                offsets.iter().for_each(|(key, offset)| {
                    self.commit_offset(key.clone(), *offset).unwrap();
                });
                runtime.reply_ok(req).await
            }
            RequestBody::ListCommittedOffsets { keys } => {
                runtime
                    .reply(
                        req,
                        ResponseBody::ListCommittedOffsetsOk {
                            offsets: keys
                                .iter()
                                .map(|key| (key.clone(), (self.get_committed_offset(key).unwrap())))
                                .collect(),
                        },
                    )
                    .await
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
enum RequestBody {
    Init {},
    Send { key: String, msg: u64 },
    Poll { offsets: HashMap<String, usize> },
    CommitOffsets { offsets: HashMap<String, usize> },
    ListCommittedOffsets { keys: Vec<String> },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum ResponseBody {
    SendOk {
        offset: usize,
    },
    PollOk {
        msgs: HashMap<String, Vec<(usize, u64)>>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>,
    },
}
