mod s7_types;
mod s7_log_record;
mod file_logger;

use std::str::FromStr;
use std::net::TcpStream;
use clap::App;

use crate::s7_types::*;
use s7_log_record::*;
use file_logger::*;

struct AppConfig {
    ip: String,
    port: String,
    filename: String,
    header: String,
    format: [S7Type; 8]
}

fn get_config(cmd_params: &clap::ArgMatches) -> AppConfig {
    let mut config = AppConfig {
        ip: "localhost".to_string(),
        port: "3000".to_string(),
        filename: "autoid_log.csv".to_string(),
        header: "time, data0, data1, data2, data3, data4, data5, data6, data7, digital".to_string(),
        format: [S7Type::Int; 8]
    };

    let mut settings = config::Config::default();

    match settings.merge(config::File::with_name("Settings")) {
        Err(e) => println!("[ERROR] {}\nTip - use commandline arguments", e),
        Ok(_) => {
            if let Ok(val) = settings.get_str("ip") {
                config.ip = val;
            }
            if let Ok(val) = settings.get_str("port") {
                config.port = val;
            }
            if let Ok(val) = settings.get_str("filename") {
                config.filename = val;
            }
            if let Ok(val) = settings.get_str("header") {
                config.header = val;
            }
            if let Ok(val) = settings.get_str("format") {
                let vec: Vec<Result<S7Type, S7TypeError>> = val.split(' ').map(|s| S7Type::from_str(s)).collect();
                //println!("{}", vec.len());
                for i in 0..8 {
                    config.format[i] = match vec[i] {
                        Ok(t) => t,
                        Err(_) => S7Type::Int,
                    };
                }
            }
        } 
    };

    if let Some(val) = cmd_params.value_of("address") {
        config.ip = val.to_string();
    }

    if let Some(val) = cmd_params.value_of("port") {
        config.port = val.to_string();
    }

    if let Some(val) = cmd_params.value_of("file") {
        config.filename = val.to_string();
    }

    config
}

fn main() -> std::io::Result<()> {
    // setup command line interface
    let cmd_params = App::new("AutoID logger")
       .version("0.1")
       .about("Logs the stream from AutoID PLC into a file")
       .author("Denis L.")
       .args_from_usage(
            "-a, --address=[IP] 'Sets an IP address to connect'
            -p, --port=[PORT] 'Sets a port to connect'
            -f, --file=[FILE] 'Sets a log file name'")
       .get_matches(); 
    
    // create configuration
    println!("Reading config");
    let config = get_config(&cmd_params);

    // Ctrl+C to terminate the program
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!. Terminating...");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Trying to connect
    println!("Connecting to {}:{}... ", config.ip, config.port);
    let mut stream = TcpStream::connect(format!("{}:{}", config.ip, config.port))?;
    println!("connected.");

    // Create log file
    let mut filelog = FileLogger::open(&config.filename)?;
    filelog.write_line(&config.header)?;

    loop {
        // read logger record with TCP
        let record = read_s7_record(&mut stream, &config.format)?;

        // get string representation
        let log_line = format!("{}", record); 

        // display 
        println!("{}", log_line);

        // push into a log file
        filelog.write_line(&log_line)?;
    }
}
