mod ring_bench;
use std::fs::File;
use std::io::prelude::*;
extern crate actix;
extern crate futures;
extern crate time;

fn main() -> std::io::Result<()> {
    let tests = ring_bench::bench::tests();

    {
        let mut a_out = File::create("results/results.actix")?;
        let mut t_out = File::create("results/results.threads")?;
        for spec in tests {
            println!("Running {}:", spec);
            print!("  actix...");
            let r = ring_bench::ring_actix::run(&spec);
            println!(" {}s.", (r.setup + r.run) / 1000000000);
            a_out.write_fmt(format_args!("{}\n", r))?;
            print!("  threads...");
            let r = ring_bench::threads::run(&spec);
            println!(" {}s.", (r.setup + r.run) / 1000000000);
            t_out.write_fmt(format_args!("{}\n", r))?;
        }
    }
    println!("done.");
    Ok(())
}
