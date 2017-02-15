use std::any::TypeId;
use std::collections::HashMap;
use std::io;
use std::thread;
use std::sync::{Arc, Weak};
use crossbeam::sync::MsQueue;

use super::{Loader, LoaderResult};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Token(TypeId);

impl Token {
    pub fn of<A: Loader>() -> Self {
        Token(TypeId::of::<A>())
    }
}

pub struct Request {
    pub token: Token,
    pub path: String,
}

impl Request {
    fn new<P: Into<String>>(token: Token, path: P) -> Self {
        Request {
            token: token,
            path: path.into(),
        }
    }
}

pub type RequestSender = Arc<MsQueue<Request>>;
pub type RequestReceiver = Weak<MsQueue<Request>>;

pub type Response<T> = Result<T, io::Error>;
pub type ResponseReceiver<T> = Arc<MsQueue<(Request, LoaderResult<T>)>>;
pub type ResponseSender<T> = Weak<MsQueue<(Request, LoaderResult<T>)>>;

pub struct RequestQueue<A: Loader> {
    sender: RequestSender,
    receiver: ResponseReceiver<A::Output>,
}

impl<A: Loader> RequestQueue<A> {
    pub fn new(sender: RequestSender, receiver: ResponseReceiver<A::Output>) -> Self {
        RequestQueue {
            sender: sender,
            receiver: receiver,
        }
    }

    pub fn send_request<P: Into<String>>(&self, path: P) {
        let request = Request::new(Token::of::<A>(), path.into());
        self.sender.push(request)
    }

    pub fn next_response(&self) -> Option<(Request, LoaderResult<A::Output>)> {
        self.receiver.try_pop()
    }

    pub fn wait_next_response(&self) -> (Request, LoaderResult<A::Output>) {
        self.receiver.pop()
    }
}

pub struct RequestHandler<A: Loader> {
    sender: ResponseSender<A::Output>,
    loader: A,
}

impl<A: Loader> RequestHandler<A> {
    pub fn new(sender: ResponseSender<A::Output>, loader: A) -> Self {
        RequestHandler {
            sender: sender,
            loader: loader,
        }
    }
}

pub trait Handler: Send {
    fn handle(&mut self, request: Request) -> bool;
}

impl<A: Loader> Handler for RequestHandler<A> {
    fn handle(&mut self, request: Request) -> bool {
        let result = self.loader.load(&request.path);

        self.sender.upgrade()
            .map(|sender| sender.push((request, result)))
            .is_some()
    }
}

pub type HandlerMap = HashMap<Token, Box<Handler>>;

pub struct LoadingThread {
    receiver: RequestReceiver,
    handlers: HandlerMap,
}

impl LoadingThread {
    pub fn new(receiver: RequestReceiver, handlers: HandlerMap) -> Self {
        LoadingThread {
            receiver: receiver,
            handlers: handlers,
        }
    }

    pub fn run(mut self) {
        thread::spawn(move || {
            while let Some(receiver) =  self.receiver.upgrade() {
                self.handle_request(receiver.pop());
            }
        });
    }

    fn handle_request(&mut self, request: Request) {
        let token = request.token;

        let result = self.handlers.get_mut(&token)
                                  .map(|handler| handler.handle(request));

        if let Some(false) = result {
            self.handlers.remove(&token);
        }
    }
}