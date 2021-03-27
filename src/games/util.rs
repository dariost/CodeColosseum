use std::collections::HashMap;
use std::str::FromStr;

pub(crate) fn arg<T: FromStr>(
    m: &HashMap<String, String>,
    a: &str,
    d: T,
) -> Result<T, <T as FromStr>::Err> {
    match m.get(a) {
        Some(x) => x.parse(),
        None => Ok(d),
    }
}

macro_rules! lnout {
    ($stream:expr, $msg:expr) => {{
        let msg = String::from($msg) + "\n";
        match $stream.write_all(msg.as_bytes()).await {
            Ok(_) => {}
            Err(x) => {
                error!("Cannot write to stream: {}", x);
                return;
            }
        }
    }};
}

macro_rules! lnin {
    ($stream:expr) => {{
        let mut s = String::new();
        match $stream.read_line(&mut s).await {
            Ok(_) => s.trim().to_string(),
            Err(x) => {
                error!("Cannot read from stream: {}", x);
                return;
            }
        }
    }};
}
