use screen::{process_file, receiver, screen::Screen};
use std::{env, error::Error, thread};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [id, orders_path] = args.as_slice() else {
        Err("Use: cargo run <id> <path_to_orders_file>")?
    };

    let screen_id: u16 = id.parse().map_err(|_| "id needs to be a number")?;
    let screen = Screen::new(screen_id);

    let receiver = {
        let screen = screen.clone();
        thread::spawn(move || receiver(screen))
    };

    process_file(orders_path.to_string(), screen, screen_id)?;

    match receiver.join() {
        Ok(Err(e)) => eprintln!("{e}"),
        Err(e) => eprintln!("{e:?}"),
        _ => {}
    }

    Ok(())
}
