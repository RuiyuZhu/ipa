use crate::query::{
    runner::QueryResult,
    state::{RemoveQuery, RunningQuery},
};
use futures::FutureExt;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Query completion polls the tokio task to get the results and cleans up the query state after.
pub struct Handle<'a> {
    _query_state_guard: RemoveQuery<'a>,
    inner: RunningQuery,
}

impl<'a> Future for Handle<'a> {
    type Output = QueryResult;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}

impl<'a> Handle<'a> {
    pub fn new(guard: RemoveQuery<'a>, inner: RunningQuery) -> Self {
        Self {
            _query_state_guard: guard,
            inner,
        }
    }
}
