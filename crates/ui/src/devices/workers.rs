use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use tokio::task::{JoinError, JoinHandle};

pub(super) fn completed<T>(workers: &mut Vec<JoinHandle<T>>) -> Vec<Result<T, JoinError>> {
    let mut results = Vec::new();
    let mut context = Context::from_waker(Waker::noop());
    workers.retain_mut(|worker| match Pin::new(worker).poll(&mut context) {
        Poll::Pending => true,
        Poll::Ready(result) => {
            results.push(result);
            false
        }
    });
    results
}
