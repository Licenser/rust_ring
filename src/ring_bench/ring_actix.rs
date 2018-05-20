use super::bench;
use actix::prelude::*;
use futures::{future, Future};
use futures::future::Future;
use actix::*;
use std::io;
struct RingActor {
    next: Option<Addr<Unsync, RingActor>>,
    id: u32,
    max: u32,
    msgs: u32,
    first: Option<Addr<Unsync, RingActor>>,
}

struct Data {
    data: String,
}
impl Message for Data {
    type Result = bool;
}

struct CloseRing {
    first: Addr<Unsync, RingActor>,
}
impl Message for CloseRing {
    type Result = bool;
}

impl Actor for RingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.id < self.max {
            self.next = Some(
                RingActor {
                    next: None,
                    first: None,
                    id: self.id + 1,
                    max: self.max,
                    msgs: self.msgs,
                }.start(),
            );
        };
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("[{}] Actor is stopped", self.id);
    }
}
impl Handler<Data> for RingActor {
    type Result = bool;

    fn handle(&mut self, msg: Data, _ctx: &mut Context<Self>) -> Self::Result {
        match self.next {
            Some(ref next) => {
                if self.id == 0 {
                    if self.msgs == 0 {
                        Arbiter::system().do_send(msgs::SystemExit(0));
                    } else {
                        self.msgs -= 1;
                        next.do_send(msg);
                    }
                } else {
                    next.do_send(msg);
                }
            }
            None => println!("[{}] Next was null!", self.id),
        };
        true
    }
}
impl Handler<CloseRing> for RingActor {
    type Result = bool;

    fn handle(&mut self, msg: CloseRing, _ctx: &mut Context<Self>) -> Self::Result {
        match self.next {
            Some(ref next) => {
                next.do_send(msg);
                ()
            }
            None => {
                self.next = Some(msg.first);
                ()
            }
        };
        true
    }
}

pub fn run(spec: &bench::Spec) -> bench::Result {
    let system = System::new("bench");

    let addr: Addr<Unsync, _> = RingActor {
        next: None,
        first: None,
        id: 0,
        max: spec.procs,
        msgs: spec.messages,
    }.start();
    //println!("sending data to start");
    let res = addr.send(CloseRing {
        first: addr.clone(),
    });
    system.handle().spawn(res.then(|res| println!("res: {}", res)));
    addr.do_send(Data{data: String::from("")});
    system.run();
    bench::Result {
        name: String::from("rust_actix"),
        spec: spec.clone(),
        setup: 0,
        run: 0,
    }
}
