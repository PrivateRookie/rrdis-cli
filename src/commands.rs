use bytes::{BufMut, BytesMut};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub enum Commands {
    /// set a key with string value
    Set {
        /// redis key
        key: String,

        /// redis key value
        value: String,

        /// set key expiration in seconds
        #[structopt(short, long)]
        ex: Option<u32>,

        /// set key expiration in milliseconds
        #[structopt(short, long)]
        px: Option<u32>,

        /// operate if key does not exist
        #[structopt(short, long)]
        nx: bool,

        /// operate if key does exist
        #[structopt(short, long)]
        xx: bool,
    },
    /// get string value
    Get {
        /// redis key
        key: String,
    },
    /// increase 1
    Incr {
        /// redis key
        key: String,
    },
    /// get list with limit range
    LRange {
        /// redis key
        key: String,

        /// start position
        start: i64,

        /// stop position
        stop: i64,
    },
    /// test server status
    Ping,
}

impl Commands {
    pub fn to_bytes(&self) -> bytes::BytesMut {
        let mut command = BytesMut::with_capacity(1024);
        match self {
            Commands::Set {
                key,
                value,
                ex,
                px,
                nx,
                xx,
            } => {
                let mut count = 3u8;
                if ex.is_some() {
                    count += 1
                }
                if px.is_some() {
                    count += 1
                }
                if *nx {
                    count += 1
                }
                if *xx {
                    count += 1
                }

                let mut args = vec![
                    format!("*{}", count),
                    "$3".to_string(),
                    "SET".to_string(),
                    format!("${}", key.len()),
                    key.clone(),
                    format!("${}", value.len()),
                    value.clone(),
                ];
                if let Some(ex) = ex {
                    args.push("$2".to_string());
                    args.push("EX".to_string());
                    args.push(format!("${}", ex.to_string().len()));
                    args.push(ex.to_string())
                }
                if let Some(px) = px {
                    args.push("$2".to_string());
                    args.push("EX".to_string());
                    args.push(format!("${}", px.to_string().len()));
                    args.push(px.to_string())
                }
                if *nx {
                    args.push("$2".to_string());
                    args.push("NX".to_string())
                }
                if *xx {
                    args.push("$2".to_string());
                    args.push("XX".to_string())
                }
                command.put(&args.join("\r\n").to_string().into_bytes()[..]);
                command.put(&b"\r\n"[..]);
            }
            Commands::Get { key } => {
                command.put(
                    &format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key).into_bytes()[..],
                );
            }
            Commands::Incr { key } => command.put(
                &format!("*2\r\n$4\r\nINCR\r\n${}\r\n{}\r\n", key.len(), key).into_bytes()[..],
            ),
            Commands::LRange { key, start, stop } => {
                let args = vec![
                    "*4".to_string(),
                    "$6".to_string(),
                    "LRANGE".to_string(),
                    format!("${}", key.len()),
                    key.clone(),
                    format!("${}", start.to_string().len()),
                    format!("{}", start),
                    format!("${}", stop.to_string().len()),
                    format!("{}", stop),
                ];
                command.put(&args.join("\r\n").to_string().into_bytes()[..]);
                command.put(&b"\r\n"[..]);
            }
            Commands::Ping => command.put(&b"*1\r\n$4\r\nPING\r\n"[..]),
        }
        log::debug!("{:?}", command);
        command
    }
}
