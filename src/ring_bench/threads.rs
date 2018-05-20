use super::bench;
use std::sync::mpsc;
use std::thread;
extern crate time;

pub fn run(spec: &bench::Spec) -> bench::Result {
    // Set up test data
    let data = (0..spec.size).map(|_| "x").collect::<String>();

    // time initialization
    let start = time::precise_time_ns();
    let (tx_first, rx) = mpsc::channel();
    let (tx, t) = init_ring(spec.procs, spec.clone(), tx_first);
    let _ = rx.recv().expect("Unable to receive from channel");
    let setup_time = time::precise_time_ns() - start;
    // At the end of the initialization we return message to measure it

    // We now start the round trime benchmark
    let start = time::precise_time_ns();
    for _ in 0..spec.paralell {
        tx.send(data.clone()).unwrap();
    }
    for _ in 0..spec.messages {
        let v = rx.recv().expect("Unable to receive from channel");
        tx.send(v).unwrap();
    }
    let end = time::precise_time_ns();
    t.join().unwrap();
    bench::Result {
        name: String::from("rust_threads"),
        spec: spec.clone(),
        setup: setup_time,
        run: end - start,
    }
}

fn init_ring(
    n: u32,
    spec: bench::Spec,
    tx_first: mpsc::Sender<String>,
) -> (mpsc::Sender<String>, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let t = thread::spawn(move || {
        if n > 1 {
            let (tx_inner, next) = init_ring(n - 1, spec.clone(), tx_first);
            for _ in 0..spec.messages {
                let v = rx.recv().expect("Unable to receive from channel");
                tx_inner.send(v).unwrap();
            }
            next.join().unwrap();
        } else {
            tx_first.send(String::from("")).unwrap();
            for _ in 0..spec.messages {
                let v = rx.recv().expect("Unable to receive from channel");
                tx_first.send(v).unwrap();
            }
        };
    });
    (tx.clone(), t)
}
