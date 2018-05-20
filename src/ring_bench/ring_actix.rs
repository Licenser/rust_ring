// The most forward implementation of using actix for a process ring
// as actix by default runs in a single thread this implementation
// does the same - all actors are on the same thread.
//
// We create one actor for each process in the ring, that actor then
// creates the next one and so on.
//
// Once all are created we send a `CloseRing` message and inform the
// actors of the first node.
// The `CloseRing` message is passed down the ring until an actor has
// no `next` actor in which case `next` is set to `first` and by that
// the ring is complete.
//
// The actors themselfs are reather simple, they get a `Data` message
// and then send a new `Data` message with the same content to the
// next element in the ring.
//
// The first actor (having `id=0`) counts how often it has seen a
// `Data` message to see how many round trips have been made.
//
// For paralellism we allow putting muliple `Data` messages on the ring.
use super::bench;
use actix::prelude::*;
use actix::*;
use time;

// The ring actor
struct RingActor {
    // Next actor in the ring - must allow None as the actor only knows
    // about the next actor once it has been created.
    next: Option<Addr<Unsync, RingActor>>,
    // The actor id (place in ring)
    id: u32,
    // Max number of actors we want
    max: u32,
    // Number of messages to pass
    msgs: u32
}

// The data message, containing a string as payload
struct Data {
    data: String,
}
impl Message for Data {
    type Result = bool;
}
// The close ring message to set the first actor as next for the last actor
struct CloseRing {
    first: Addr<Unsync, RingActor>,
}
impl Message for CloseRing {
    type Result = bool;
}

// Actor implementation
impl Actor for RingActor {
    type Context = Context<Self>;
 
    fn started(&mut self, _ctx: &mut Self::Context) {
        // As long as the actor id is smaller then the max
        // number of actors create a new actor with the next id
        if self.id < self.max {
            self.next = Some(
                RingActor {
                    next: None,
                    id: self.id + 1,
                    max: self.max,
                    msgs: self.msgs,
                }.start(),
            );
        };
    }
}

// Implementation of the handler for Data message
impl Handler<Data> for RingActor {
    type Result = bool;
    fn handle(&mut self, msg: Data, _ctx: &mut Context<Self>) -> Self::Result {
        // We make sure that we have a next
        match self.next {
            Some(ref next) => {
                // If we do we check if we are the first process
                if self.id == 0 {
                    // If so we know we can end our system if we have send our
                    // messages
                    if self.msgs == 0 {
                        Arbiter::system().do_send(msgs::SystemExit(0));
                    } else {
                        // Otherwise we decrement the message count and send a new
                        // `Data` message to the next actor in the ring.
                        self.msgs -= 1;
                        next.do_send(Data{data: msg.data});
                    }
                } else {
                    // If we are not the first process we just keep passing on
                    // `Data` messages
                    next.do_send(Data{data: msg.data});
                }
            }
            // This should never happen, we have no next item in then ring
            // so if it does we panic!
            None => panic!("[{}] Next was null! This is not a ring it's a string :(", self.id),
        };
        true
    }
}
// Implementation of the `CloseRing`
impl Handler<CloseRing> for RingActor {
    type Result = bool;

    fn handle(&mut self, msg: CloseRing, _ctx: &mut Context<Self>) -> Self::Result {
        // We match on next
        match self.next {
            // If we have a next we pass this message on to the next actor
            Some(ref next) => {
                next.do_send(msg);
                ()
            }
            // If not weŕe the last actor and set the first node as our next node
            None => {
                self.next = Some(msg.first);
                ()
            }
        };
        true
    }
}

pub fn run(spec: &bench::Spec) -> bench::Result {
    // Pre-generate the payload so weŕe not measuring string
    // creation.
    let data = (0..spec.size).map(|_| "x").collect::<String>();
    let start = time::precise_time_ns();
    // Start a new System.
    let system = System::new("bench");
    // Create our first actor
    let addr: Addr<Unsync, _> = RingActor {
        next: None,
        id: 0,
        max: spec.procs,
        msgs: spec.messages,
    }.start();
    // Since the first actor will create the second and so one at this
    // point our ring is nearly complete it just needs to be closed.
    addr.do_send(CloseRing {
        first: addr.clone(),
    });
    let setup_time = time::precise_time_ns() - start;
    let start = time::precise_time_ns();
    // Next we put Data messages on the ring limited by the number
    // of paralell messages we want.
    for _ in 0..spec.paralell {
        addr.do_send(Data{data: data.clone()});
    };
    // The ring will run until our first actor decides it's time to shut down.
    system.run();
    let end = time::precise_time_ns();
    bench::Result {
        name: String::from("rust_actix"),
        spec: spec.clone(),
        setup: setup_time,
        run: end - start,
    }
}
