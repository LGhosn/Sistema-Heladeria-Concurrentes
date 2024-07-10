use gateway::gateway::Gateway;
use ice_cream_shop::shop_values::GATEWAY_PORT;

fn main() {
    let gate_way = Gateway::new(GATEWAY_PORT);
    match gate_way.receive_messages() {
        Ok(_) => println!("Gateway is running"),
        Err(e) => eprintln!("Error running gateway: {}", e),
    }
}
