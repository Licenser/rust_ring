use std::fmt;
#[derive(Debug, Clone)]
pub struct Spec {
    pub procs: u32,
    pub messages: u32,
    pub paralell: u32,
    pub size: u32,
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} procs with {} messages {} in paralell of a size of {}",
            self.procs, self.messages, self.paralell, self.size
        )
    }
}

#[derive(Debug, Clone)]
pub struct Result {
    pub name: String,
    pub spec: Spec,
    pub setup: u64,
    pub run: u64,
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{}",
            self.name,
            self.spec.procs,
            self.spec.messages,
            self.spec.paralell,
            self.spec.size,
            self.setup,
            self.run
        )
    }
}
pub fn tests() -> Vec<Spec> {
    let max = 4;
    let mut v = Vec::new();
    for procs in 1..max {
        for msgs in 1..max {
            for paralell in 0..(max + 1) {
                for size in 1..(max + 1) {
                    v.push(Spec {
                        procs: 10_u32.pow(procs),
                        messages: 10_u32.pow(msgs),
                        paralell: 2_u32.pow(paralell),
                        size: 10_u32.pow(size),
                    })
                }
            }
        }
    }
    v
}
