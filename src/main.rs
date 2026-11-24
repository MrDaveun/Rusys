mod system;
mod hardware;
mod network;

fn main() {
    let info = system::collect_info();
    println!("{}", info.os);
    println!("{}", info.arch);
    println!("{}", info.family);
    println!("{}", info.hostname.expect("Hostname unavailable"));
}
