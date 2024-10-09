//! Implementation of a [`Future`] that concurrently awaits two sends, and informs the caller of the one which failed,
//! if any.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{ready, Sink};

use crate::game::Side;

/// A [`Future`] completing when both sends have succeeded, or returning which [`Sink`] failed.
pub struct CombinedSend<'a, Si: ?Sized, Item> {
    left_sink: &'a mut Si,
    right_sink: &'a mut Si,
    item: Option<Item>,
}

impl<'a, Si, Item> CombinedSend<'a, Si, Item> {
    pub fn new(left_sink: &'a mut Si, right_sink: &'a mut Si, item: Item) -> Self {
        Self {
            left_sink,
            right_sink,
            item: Some(item),
        }
    }
}

impl<Si: Unpin, Item> Unpin for CombinedSend<'_, Si, Item> {}

impl<Si, Item> Future for CombinedSend<'_, Si, Item>
where
    Si: Sink<Item> + Unpin,
    Item: Clone,
{
    type Output = Result<(), (Si::Error, Side)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let (mut left_sink, mut right_sink) = (
            Pin::new(&mut this.left_sink),
            Pin::new(&mut this.right_sink),
        );

        if this.item.is_some() {
            let (left_ready, right_ready) = (
                left_sink.as_mut().poll_ready(cx),
                right_sink.as_mut().poll_ready(cx),
            ); //TODO don't return of one pending
            if let Err(e) = ready!(left_ready) {
                return Poll::Ready(Err((e, Side::Left)));
            }
            if let Err(e) = ready!(right_ready) {
                return Poll::Ready(Err((e, Side::Right)));
            }

            let item = this
                .item
                .take()
                .expect("polled CombinedSend after completion");
            if let Err(e) = left_sink.as_mut().start_send(item.clone()) {
                return Poll::Ready(Err((e, Side::Left)));
            }
            if let Err(e) = right_sink.as_mut().start_send(item) {
                return Poll::Ready(Err((e, Side::Right)));
            }
        }

        let (left_flush, right_flush) = (
            left_sink.as_mut().poll_flush(cx),
            right_sink.as_mut().poll_flush(cx),
        );
        if let Err(e) = ready!(left_flush) {
            return Poll::Ready(Err((e, Side::Left)));
        }
        if let Err(e) = ready!(right_flush) {
            return Poll::Ready(Err((e, Side::Right)));
        }

        Poll::Ready(Ok(()))
    }
}
