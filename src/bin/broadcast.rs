use anyhow::Context;
use rustagen::*;
use std::io::StdoutLock;
use std::{collections::HashMap, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            id: 1,
            node: init.node_id,
            messages: Vec::new(),
        })
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                reply.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to boradcast")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }

            Payload::Read => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to boradcast")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }

            Payload::Topology { .. } => {
                reply.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to boradcast")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }

            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }

        Ok(())
    }
}
fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
