mod ring_bench;
use std::fs::File;
use std::io::prelude::*;
extern crate actix;
extern crate futures;

fn main() -> std::io::Result<()> {
    let tests = ring_bench::bench::tests();

    {
        let mut out = File::create("results/results.actix")?;
        for spec in tests {
            println!("Running actix {}.", spec);
            let r = ring_bench::ring_actix::run(&spec);
            out.write_fmt(format_args!("{}\n", r))?;
            break;
        }
    }
    println!("done.");

    // {
    //     let mut out = File::create("results/results.threads")?;
    //     for spec in tests {
    //         println!("Running threads {}.", spec)
    //         let r = ring_bench::threads::run(&spec);
    //         out.write_fmt(format_args!("{}\n", r))?;
    //     }
    // }
    // println!("done.");
    Ok(())
}
