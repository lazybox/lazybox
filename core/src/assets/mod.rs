mod request;

use std::io;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use self::request::*;
use std::sync::Arc;
use sync::MsQueue;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct AssetRef(String);

impl AssetRef {
    pub fn new<P: Into<String>>(path: P) -> Self {
        AssetRef(path.into())
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

pub type LoaderResult<T> = Result<T, io::Error>;

pub trait Loader: Any + Send {
    type Output: Send;

    fn load(&mut self, path: &str) -> LoaderResult<Self::Output>;
}

pub enum Status {
    Loading,
    Available,
}

type StatusMap = HashMap<AssetRef, Status>;

pub struct AsyncLoader<A: Loader> {
    queue: RequestQueue<A>,
    status: StatusMap,
    pending_requests: u32,
}

impl<A: Loader> AsyncLoader<A> {
    fn new(queue: RequestQueue<A>) -> Self {
        AsyncLoader {
            queue: queue,
            status: StatusMap::new(),
            pending_requests: 0,
        }
    }

    pub fn load(&mut self, assert_ref: &AssetRef) {
        use std::collections::hash_map::Entry;

        match self.status.entry(assert_ref.clone()) {
            Entry::Vacant(vacant) => {
                vacant.insert(Status::Loading);
                self.queue.send_request(assert_ref.path());
                self.pending_requests += 1;
            }
            Entry::Occupied(occupied) => {
                let status = occupied.into_mut();

                if let Status::Available = *status {
                    self.queue.send_request(assert_ref.path());
                    self.pending_requests += 1;
                }
            }
        }
    }

    pub fn status(&self, asset_ref: &AssetRef) -> Option<&Status> {
        self.status.get(asset_ref)
    }

    pub fn ready_assets(&mut self) -> ReadyAssetIter<A> {
        ReadyAssetIter { async_loader: self }
    }

    pub fn wait_for_ready(&mut self, mut amount: u32) -> WaitAssetIter<A> {
        if amount > self.pending_requests {
            amount = self.pending_requests;
        }
        let limit = self.pending_requests - amount;

        WaitAssetIter {
            async_loader: self,
            limit: limit,
        }
    }

    pub fn wait_all(&mut self) -> WaitAssetIter<A> {
        WaitAssetIter {
            async_loader: self,
            limit: 0,
        }
    }

    fn handle_response(request: Request,
                       result: LoaderResult<A::Output>,
                       status: &mut StatusMap)
                       -> (AssetRef, LoaderResult<A::Output>) {

        let asset_ref = AssetRef::new(request.path);
        if result.is_ok() {
            status.insert(asset_ref.clone(), Status::Available);
        }

        (asset_ref, result)
    }
}

pub struct ReadyAssetIter<'a, A: Loader> {
    async_loader: &'a mut AsyncLoader<A>,
}

impl<'a, A: Loader> Iterator for ReadyAssetIter<'a, A> {
    type Item = (AssetRef, LoaderResult<A::Output>);

    fn next(&mut self) -> Option<Self::Item> {
        let AsyncLoader::<A> { ref mut queue, ref mut status, ref mut pending_requests } =
            *self.async_loader;

        queue.next_response().map(|(request, result)| {
            *pending_requests -= 1;
            AsyncLoader::<A>::handle_response(request, result, status)
        })
    }
}

pub struct WaitAssetIter<'a, A: Loader> {
    async_loader: &'a mut AsyncLoader<A>,
    limit: u32,
}

impl<'a, A: Loader> Iterator for WaitAssetIter<'a, A> {
    type Item = (AssetRef, LoaderResult<A::Output>);

    fn next(&mut self) -> Option<Self::Item> {
        let AsyncLoader::<A> { ref mut queue, ref mut status, ref mut pending_requests } =
            *self.async_loader;

        if self.limit == *pending_requests {
            None
        } else {
            *pending_requests -= 1;
            let (request, result) = queue.wait_next_response();
            Some(AsyncLoader::<A>::handle_response(request, result, status))
        }
    }
}

pub struct Initializer {
    request_sender: RequestSender,
    request_receiver: RequestReceiver,
    handlers: HandlerMap,
}

impl Initializer {
    fn new() -> Self {
        let queue = Arc::new(MsQueue::new());

        Initializer {
            request_sender: queue.clone(),
            request_receiver: Arc::downgrade(&queue),
            handlers: HandlerMap::new(),
        }
    }

    pub fn create_async_loader<A: Loader>(&mut self, loader: A) -> AsyncLoader<A> {
        let queue = Arc::new(MsQueue::new());
        let handler = RequestHandler::new(Arc::downgrade(&queue), loader);

        self.handlers.insert(Token::of::<A>(), Box::new(handler));

        let queue = RequestQueue::new(self.request_sender.clone(), queue);
        AsyncLoader::new(queue)
    }

    pub fn done(self) {
        let thread = LoadingThread::new(self.request_receiver, self.handlers);
        thread.run();
    }
}

pub fn init() -> Initializer {
    static ALREADY_RUN: AtomicBool = ATOMIC_BOOL_INIT;

    if ALREADY_RUN.swap(true, Ordering::SeqCst) {
        panic!("You can only init assets crate once per run.");
    }

    Initializer::new()
}