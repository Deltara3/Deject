use std::env;
use deject::injector::Injector;

fn main() {
    // Temporary test code, hence the unsafe stuff.
    let argv = env::args().collect::<Vec<String>>();
    let pid  = argv[1].parse::<u32>().unwrap();
    let dll  = argv[2].clone();

    let injector = Injector::from_pid(pid, false).unwrap();

    if injector.is_uwp() {
        Injector::fix_access_control(&dll);
    }

    injector.inject_dll(&dll).unwrap();
}
