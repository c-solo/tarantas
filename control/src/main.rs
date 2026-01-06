use control::network::serial::SerialConnection;

fn main() {
    let _net = SerialConnection::new("/dev/ttyUSB0", 115200);

    println!("Robot control software");
}
