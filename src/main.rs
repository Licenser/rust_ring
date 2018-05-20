mod ring_bench;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let tests = ring_bench::bench::tests();
    {
        let mut out = File::create("results/results.threads")?;
        for spec in tests {
            if spec.procs > spec.paralell {
                println!("Running threads {}.", spec);
                let r = ring_bench::threads::run(&spec);
                out.write_fmt(format_args!("{}\n", r))?;
            }
        }
    }
    println!("done.");
    Ok(())
}
