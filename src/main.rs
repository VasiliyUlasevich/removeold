pub use libc;

use std::env;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::prelude::*;
use chrono::prelude::*;

const VER: &str = "0.1";

fn write_log_message(log_file: &mut std::fs::File , message: &str) {
    let local_time: DateTime<Local> = Local::now();
    match log_file.write_fmt(format_args!("{} {}\n",local_time.format("%Y-%m-%d %H:%M:%S").to_string(), message)) {
        Ok(r) => r,
        Err(err) => {
            println!("Error writing to log file: {}", err.to_string());
            std::process::exit(-1);
        }
    };
}


fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut ssize: &str;
    let mut log_write_flag = false;
    let mut log: String = String::from("");
    let mut ext: String = String::from("");
    let mut size: usize = 0;
    let mut log_file: Option<std::fs::File> = None;

    println!("RemoveOLD v{}", VER);
    args.remove(0); // remove programm name
    for p in args {
        let t_arg: Vec<&str> = p.split('=').collect();
        match t_arg[0] {
            "ext" => {
                if t_arg.len() < 2 {
                    println!("You should specify value of the 'ext' parameter!");
                    return;
                }
                ext = String::from(t_arg[1]);
            },
            "size" => {
                if t_arg.len() < 2 {
                    println!("Invalid value of the 'size' parameter!");
                    return;
                }
                ssize = t_arg[1];
                let l = ssize.len();
                if l == 0 {
                    println!("Invalid value of the 'size' parameter!");
                    return;
                } else {
                    let mult:usize;
                    match ssize.chars().last().unwrap() {
                        '0'..='9' => mult = 1,
                        'K' | 'k' => mult = 1024,
                        'M' | 'm' => mult = 1024*1024,
                        'G' | 'g' => mult = 1024*1024*1024,
                        _ => {
                            println!("Invalid value of the 'size' parameter!");
                            return;
                        }
                    }
                    let t;
                    if mult == 1 {
                        t = ssize[0..l].parse::<usize>();

                    } else {
                        t = ssize[0..l-1].parse::<usize>();
                    }
                    if t.is_err() {
                        println!("Invalid value of the 'size' parameter!");
                        return;
                    } else {
                        size = t.unwrap() * mult;
                    }
                }
            },
            "log" =>{
                if t_arg.len() < 2 {
                    println!("You should specify value of the 'log' parameter!");
                    return;
                }
                log = String::from(t_arg[1]);
                log_write_flag = true;
            },
            "help" | "-?" | "-help" => {
                // TODO: show help and validate it
                println!("Command line arguments:");
                println!("  help - show this help");
                println!("  size=number - if number is greater then free space, the program will search for and deleting the oldest files. This parameter is required.");
                println!("  ext=extension - if specify this parameter, the application will only look for files with that extension.");
                println!("  log=log_file_name - if specify this parameter, the application will save the names of deleted files to this log file.");
                return;
            },
            _ => {
                println!("Invalid parameter: {}",t_arg[0]);
                return;
            },
        }
    }
    println!("Minimal size of free space: {} bytes", size);
    if size == 0 {
        println!("You should specify the 'size' parameter in command line!");
        return;
    }

    // TODO: open log file
    if log_write_flag {
        log_file = match OpenOptions::new().read(true).write(true).create(true).open(log.clone()){
            Ok(l) => Some(l),
            Err(err) => {
                println!("Error opening log file: {}", err.to_string());
                return;
            }
        };
        let mut t = log_file.expect("Something goes wrong!");
        log_file = match t.seek(SeekFrom::End(0)) {
            Ok(_) => Some(t),
            Err(err) => {
                println!("Error working with log file: {}", err.to_string());
                return;
            }
        };
        let mut t = log_file.expect("Something goes wrong!");
        write_log_message(&mut t, "Application started");
        log_file = Some(t);
    }

    // TODO: analyze space size
    let cwd = CString::new(env::current_dir().unwrap().as_os_str().as_bytes()).unwrap();
    let free_space: usize;
    unsafe {
        let mut stat_buf: libc::statvfs = mem::zeroed();
        libc::statvfs(cwd.as_ptr() as *const _, &mut stat_buf);
        free_space = stat_buf.f_bavail as usize * stat_buf.f_frsize as usize;
    }
    println!("free space: {} bytes", free_space);

    // TODO: find files to delete
    if free_space < size {

    }
    // TODO: finish

    if log_write_flag {
        let mut t = log_file.expect("Something goes wrong!");
        write_log_message(&mut t, "Application finished");
    }
    // TODO: remove those lines
    println!("ext: {}", ext);
    println!("log: {}", log);
}
