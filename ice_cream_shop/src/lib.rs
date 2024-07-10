pub mod flavour;

pub mod orders;

pub mod shop_values;

pub mod tokens;

pub mod messages;

#[macro_export]
macro_rules! io_err {
    ($msg:literal) => {
        io::Error::new(
            io::ErrorKind::Other,
            format!("{} - {}:{}", $msg, file!(), line!()),
        )
    };
}

/// Converts an id to an address.
pub fn id_to_addr(start: u16, id: u16) -> String {
    format!("127.0.0.1:{}", start + id)
}
