// A stright forward thread implementation of the ring message passing benchmark.
// This is obviously not a fair benchmark as it is well known that OS threads
// do not scale well. Non the less it is good to have as a baseline of what the
// alternative would be.
//
// The implementation is rather simplistic. From the main thread we spawn another
// thread and from that thread another and another and so on until we have
// reached the expected number of processes(threads).
//
// When creating a new thread we return the TX side of a channel so that the
// spawning process can send messages to it's 'child'. To make it a full ring
// we pass along the TX of the main thread so that our last thread can user
// this.
//
// To send data we use three messages:
// * InitDone - send by the last thread to the main thread to indicate the
//              ring is set up.
// * Terminate - send around the ring to indicate the treads are done and
//               should terminate.
// * Data{data} - a payload message with a data string to simulate size.

use super::bench;
use std::sync::mpsc;
use std::thread;
use time;
#[derive(Clone)]
enum Message {
    InitDone,
    Data { data: String },
    Terminate,
}

pub fn run(spec: &bench::Spec) -> bench::Result {
    // Set up test data
    let data = Message::Data {
        data: (0..spec.size).map(|_| "x").collect::<String>(),
    };

    // time initialization
    let start = time::precise_time_ns();
    let (tx_first, rx) = mpsc::channel();
    let (tx, t) = init_ring(spec.procs, tx_first);
    let _ = rx.recv().expect("Unable to receive from channel");
    let setup_time = time::precise_time_ns() - start;
    // At the end of the initialization we return message to measure it

    // We now start the round trime benchmark
    let start = time::precise_time_ns();

    // We initialize with a given number of messages so that they might
    // traverse our ring in paralell. We clone the data as in a realistic
    // environment we not send the exact same message multiple times.
    for _ in 0..spec.paralell {
        tx.send(data.clone()).unwrap();
    }
    // When we get a message we will just pass it back to the ring.
    // This means all our messages will be doing the rounds.
    // We do this as often as specified in the benchmark specs.
    // We might get to the situation of a handfull of messages not
    // processed at the end of the test but that's OK.
    for _ in 0..spec.messages {
        let v = rx.recv().expect("Unable to receive from channel");
        tx.send(v).unwrap();
    }
    let end = time::precise_time_ns();
    // Finally we send the terminate message to tear down the ring
    // then join the thread to wait for it to unfold.
    tx.send(Message::Terminate {}).unwrap();
    t.join().unwrap();
    bench::Result {
        name: String::from("rust_threads"),
        spec: spec.clone(),
        setup: setup_time,
        run: end - start,
    }
}

// Initializes a ring thread. This will be called recursively from
// each thread to initialize the next one.
// * `n` is the remaining number o threads to start.
// * `tx_first` is the TX side of the main thread.
fn init_ring(
    n: u32,
    tx_first: mpsc::Sender<Message>,
) -> (mpsc::Sender<Message>, thread::JoinHandle<()>) {
    // create the channel for this thread
    let (tx, rx) = mpsc::channel();
    let t = thread::spawn(move || {
        // We have to differentiate between the last thread and all
        // the rest.
        if n > 1 {
            // if `n > 1` this is not the
            // last thread and we need to set up another step and at
            // the end join the thread to make sure we have a clean
            // shutdown.
            let (tx_inner, next) = init_ring(n - 1, tx_first);
            thread_loop(rx, tx_inner);
            next.join().unwrap();
        } else {
            // otherwise we are in the last thread, we use tx_first
            // to send messages back to the beginning of the ring
            // and don't join anything at the end.
            tx_first.send(Message::InitDone {}).unwrap();
            thread_loop(rx, tx_first);
        };
    });
    (tx.clone(), t)
}

// Inner loop of each thread. When we get a `Data` message we keep sending
// the data down the ring, if we get a Terminate message we hand it on and
// stop. We panic on InitDone as that should never ever be received by
// anything but the main thread.
//
// Note that we do construct a new message on the send to at least
// semi simulate some work. This might however be optimized away? Who knows!
fn thread_loop(rx: mpsc::Receiver<Message>, tx: mpsc::Sender<Message>) {
    loop {
        match rx.recv().expect("Unable to receive from channel") {
            Message::Data { data } => tx.send(Message::Data { data: data }).unwrap(),
            Message::Terminate => {
                tx.send(Message::Terminate {}).unwrap();
                break;
            }
            Message::InitDone => panic!("We should never get InitDone in a thread!"),
        }
    }
}
