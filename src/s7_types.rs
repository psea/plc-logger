// Extend TcpStream with methods to read dwords as S7 types
use chrono::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;
use std::io::prelude::*;

pub trait S7Read {
    fn read_s7_dword_as_int(&mut self) -> Result<f64, std::io::Error>;
    fn read_s7_dword_as_real(&mut self) -> Result<f64, std::io::Error>;
    fn read_s7_time(&mut self) -> Result<DateTime<Utc>, std::io::Error>;
}

impl S7Read for TcpStream {
    fn read_s7_dword_as_int(&mut self) -> Result<f64, std::io::Error> {
        let mut buf4 = [0; 4];
        self.read_exact(&mut buf4)?;
        Ok(i32::from_be_bytes(buf4) as f64)
    }

    fn read_s7_dword_as_real(&mut self) -> Result<f64, std::io::Error> {
        let mut buf4 = [0; 4];
        self.read_exact(&mut buf4)?;
        Ok(f32::from_be_bytes(buf4) as f64)
    }

    fn read_s7_time(&mut self) -> Result<DateTime<Utc>, std::io::Error> {
        let mut buf8 = [0; 8];

        fn bcd_to_dec(b: u8) -> u8 { 10 * (b >> 4) + (b & 0xF) };
        fn bcd_to_dec_high(b: u8) -> u8 { b >> 4 };

        // read timestamp
        self.read_exact(&mut buf8)?;

        let year    = (bcd_to_dec(buf8[0]) as u32 + 2000) as i32;
        let month   = bcd_to_dec(buf8[1]) as u32;
        let day     = bcd_to_dec(buf8[2]) as u32;
        let hour    = bcd_to_dec(buf8[3]) as u32;
        let minute  = bcd_to_dec(buf8[4]) as u32;
        let second  = bcd_to_dec(buf8[5]) as u32;
        let msec    = bcd_to_dec(buf8[6]) as u32 * 10 + bcd_to_dec_high(buf8[7]) as u32;
                
        //let dt = Utc.ymd(year, month, day).and_hms_milli(hour, minute, second, msec);
        let dt = match Utc.ymd_opt(year, month, day) {
            chrono::offset::LocalResult::Single(ymd) => ymd.and_hms_milli_opt(hour, minute, second, msec),
            chrono::offset::LocalResult::Ambiguous(_,_) => None,
            chrono::offset::LocalResult::None => None
        };
        
        match dt {
            Some(d) => Ok(d),
            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Timestamp"))
        }
    }
}

// S7 type of log record format
#[derive(Copy, Clone)]
pub enum S7Type {
    Int,
    Real
}

pub enum S7TypeError {TypeParseError}

impl FromStr for S7Type {
    type Err = S7TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
           "int" => Ok(S7Type::Int),
           "real" => Ok(S7Type::Real),
           _ => Err(S7TypeError::TypeParseError)
        } 
    }
}
