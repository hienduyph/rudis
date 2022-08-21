use crate::{Connection, Db, Frame, Parse, ParseError};

use bytes::Bytes;
use std::time::Duration;
use tracing::{debug, instrument};


#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes, expire: Option<Duration>) -> Set {
        Set {
            key: key.to_string(),
            value,
            expire,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }

    pub fn expire(&self) -> Option<Duration> {
        self.expire
    }

    pub(crate) fn parse_frame(parse: &mut Parse) -> crate::Result<Set> {
        use ParseError::EndOfStream;

        // Read the set key
        let key = parse.next_string()?;

        let value = parse.next_bytes()?;

        let mut expire = None;

        match parse.next_string() {
            Ok(s) if s.to_uppercase() == "EX" => {
                // an expiration is specified in seconds. the next value is an integer
                let secs = parse.next_int()?;
                expire = Some(Duration::from_secs(secs));
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                // millis
                let millis = parse.next_int()?;
                expire = Some(Duration::from_millis(millis));
            }

            Ok(_) => return Err("currently `SET` only uspport the expiration option".into()),

            Err(EndOfStream) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(Set {key, value, expire})
    }


    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        db.set(self.key, self.value, self.expire);

        let response = Frame::Simple("OK".to_string());
        debug!(?response);
        dst.write_frame(&response).await?;
        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_bulk(Bytes::from("set".as_bytes()));
        frame.push_bulk(Bytes::from(self.key.into_bytes()));
        frame.push_bulk(self.value);

        if let Some(ms) = self.expire {
            frame.push_bulk(Bytes::from("px".as_bytes()));
            frame.push_int(ms.as_millis() as u64);
        }
        frame

    }
}