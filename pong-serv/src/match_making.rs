//! Provides match-making functionality
//!
//! Each connection accepted on the server gets its own task spawned. The purpose of this mod is to get connections in a
//! same task, so they can play together.
//!
//! The implemented logics are :
//! * Pairing incoming players together to play a game. This is done in [`join_opponents`] using a
//!   server-wide [`Mutex`].

use std::sync::Mutex;

use tokio::sync::oneshot;

pub use opponents_joining::join_opponents;

use crate::match_making::opponents_joining::GiverToExecutorData;

mod opponents_joining;

/// A server-wide structure, opaque from outside this mod but easily created from the main function, containing the
/// necessary resources to match-make and join connections handled from the many asynchronous tasks.
///
/// # Safety
///
/// The structure can't be copied nor cloned. It must be stored in an [`Arc`]. It is [`Send`], and has all the
/// inter-task synchronization primitives necessary for the mod to do its job.
pub struct MatchMaker<S> {
    mutex: Mutex<Option<oneshot::Sender<GiverToExecutorData<S>>>>,
}

impl<S> MatchMaker<S> {
    /// Creates a new [`MatchMaker`] instance.
    pub fn new() -> MatchMaker<S> {
        MatchMaker {
            mutex: Mutex::new(None),
        }
    }
}
