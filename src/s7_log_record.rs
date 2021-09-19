use std::fmt;
use std::net::TcpStream;
use chrono::prelude::*;
use crate::s7_types::*;

pub struct StreamRecord {
    datetime: DateTime<Utc>,
    analog: [f64; 8],
    digital: u32,
}

impl fmt::Display for StreamRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s;

        // add datetime
        let date_time = self.datetime.format("%H:%M:%S%.3f").to_string();
        s = format!("{}, ", date_time);

        // add analog values
        for v in self.analog.iter() {
            s.push_str(&format!("{}, ", v));
        }

        // add boolean values
//         log_line.push_str(&format!("{:032b}, ", record.digital));
        let bits:Vec<u32> = (0..31).map(|b| self.digital >> b & 1).collect();
        for b in 0..8 {
            s.push_str(&format!("{}, ", bits[b]));
        }

        write!(f, "{}", s)
    }
}

pub fn read_s7_record(stream: &mut TcpStream, types: &[S7Type; 8]) -> Result<StreamRecord, std::io::Error> {
    // traverse the stream until valid timestamp
    let mut dt;
    loop {
        dt = stream.read_s7_time();
        match dt {
            Ok(_) => break,
            Err(e) => match e.kind() {
                std::io::ErrorKind::InvalidData => {
                    println!("[ERROR] {}", e);
                    continue;
                    //dt = Ok(Utc.ymd(0, 1, 1).and_hms_milli(0, 0, 0, 0));
                    //break;
                },
                _ => return Err(e)
            }
        };
    };
    
    let mut analog: [f64; 8] = [0.0; 8];
    let digital: u32;

    for i in 0..8 {
        analog[i] = match types[i] {
            S7Type::Int => stream.read_s7_dword_as_int()?,
            S7Type::Real => stream.read_s7_dword_as_real()?,
        }
    }

    digital = stream.read_s7_dword_as_int()? as u32;

    Ok(StreamRecord {
        datetime: dt.unwrap(),
        analog: analog,
        digital: digital
    })
}
