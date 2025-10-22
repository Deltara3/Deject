use std::env;
use deject::injector::Injector;

fn main() {
    // Temporary test code, hence the unsafe stuff.
    let argv = env::args().collect::<Vec<String>>();
    let pid  = argv[1].parse::<u32>().unwrap();

    let injector = Injector::from_pid(pid).unwrap();
    println!("UWP: {}", injector.is_uwp());
}
