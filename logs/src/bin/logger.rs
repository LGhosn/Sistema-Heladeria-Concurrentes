use logs::logger_receiver::LoggerReceiver;

fn main() {
    let mut logger = LoggerReceiver::new(8080, std::io::stdout(), true);
    match logger.receive_logs() {
        Ok(_) => println!("Logger finalizado correctamente"),
        Err(e) => eprintln!("Error en el logger: {:?}", e),
    }
}
