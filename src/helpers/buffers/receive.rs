use crate::helpers::fabric::ChannelId;
use crate::helpers::{MessagePayload, MESSAGE_PAYLOAD_SIZE_BYTES};
use crate::protocol::RecordId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tokio::sync::oneshot;

/// Local buffer for messages that are either awaiting requests to receive them or requests
/// that are pending message reception.
/// TODO: Right now it is backed by a hashmap but `SipHash` (default hasher) performance is not great
/// when protection against collisions is not required, so either use a vector indexed by
/// an offset + record or [xxHash](https://github.com/Cyan4973/xxHash)
#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ReceiveBuffer {
    inner: HashMap<ChannelId, HashMap<RecordId, ReceiveBufItem>>,
    record_ids: HashMap<ChannelId, RecordId>,
}

#[derive(Debug)]
enum ReceiveBufItem {
    /// There is an outstanding request to receive the message but this helper hasn't seen it yet
    Requested(oneshot::Sender<MessagePayload>),
    /// Message has been received but nobody requested it yet
    Received(MessagePayload),
}

impl ReceiveBuffer {
    /// Process request to receive a message with the given `RecordId`.
    pub fn receive_request(
        &mut self,
        channel_id: ChannelId,
        record_id: RecordId,
        sender: oneshot::Sender<MessagePayload>,
    ) {
        match self.inner.entry(channel_id).or_default().entry(record_id) {
            Entry::Occupied(entry) => match entry.remove() {
                ReceiveBufItem::Requested(_) => {
                    panic!("More than one request to receive a message for {record_id:?}");
                }
                ReceiveBufItem::Received(payload) => {
                    sender.send(payload).unwrap_or_else(|_| {
                        tracing::warn!("No listener for message {record_id:?}");
                    });
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(ReceiveBufItem::Requested(sender));
            }
        }
    }

    /// Process messages that has been received. It assumes messages arriving in order, so first
    /// chunk will belong to range of records [0..chunk.len()), second chunk [chunk.len()..2*chunk.len())
    /// etc. It does not require all chunks to be of the same size, this assumption is baked in
    /// send buffers.
    pub fn receive_messages(&mut self, channel_id: &ChannelId, messages: &[u8]) {
        let offset = self
            .record_ids
            .entry(channel_id.clone())
            .or_insert_with(|| RecordId::from(0_u32));

        for msg in messages.chunks(MESSAGE_PAYLOAD_SIZE_BYTES) {
            let payload = msg.try_into().unwrap();
            match self
                .inner
                .entry(channel_id.clone())
                .or_default()
                .entry(*offset)
            {
                Entry::Occupied(entry) => match entry.remove() {
                    ReceiveBufItem::Requested(s) => {
                        s.send(payload).unwrap_or_else(|_| {
                            tracing::warn!("No listener for message {:?}", offset);
                        });
                    }
                    ReceiveBufItem::Received(_) => {
                        panic!("Duplicate message for the same record {:?}", offset)
                    }
                },
                Entry::Vacant(entry) => {
                    entry.insert(ReceiveBufItem::Received(payload));
                }
            };

            *offset += 1;
        }
    }
}
