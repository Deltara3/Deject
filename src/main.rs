use std::env;
use deject::injector::Injector;

fn main() {
    // Temporary test code, hence the unsafe stuff.
    let argv = env::args().collect::<Vec<String>>();
    let pid  = argv[1].parse::<u32>().unwrap();

    let _injector = Injector::from_pid(pid).unwrap();
}
