pub use libc;

use std::env;
use std::mem;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::prelude::*;
use chrono::prelude::*;
use std::path::Path;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;
use std::ffi::CString;

const VER: &str = "1.0.1";

struct Files {
    name: String,
    create_date: SystemTime,
    size: usize
}

struct FileOrNone {
    f: Option<std::fs::File>
}
impl FileOrNone {
    fn borrow_mut(&mut self) -> Result<&mut std::fs::File, &'static str> {
        match &mut self.f {
            Some(x) => Ok(x),
            None => {
                panic!("Cannot borrow None value");
            }
        }
    }
    fn write_log_message(&mut self, message: &str) {
        match &mut self.f {
            Some(x) => {
                let local_time: DateTime<Local> = Local::now();
                x.write_fmt(format_args!("{} {}\n",local_time.format("%Y-%m-%d %H:%M:%S").to_string(), message)).expect("Error writing to log file");
            },
            None => {
                panic!("Cannot borrow None value");
            }
        };
    }
}

fn get_files_list(_dir: &Path, ext: &String, _list: &mut Vec<Files>){
    if _dir.is_dir() {
        for entry in fs::read_dir(_dir).expect("Directory traversal error") {
            let entry = entry.expect("Directory read error");
            let path = entry.path();
            if path.is_dir() {
                get_files_list(&path, &ext, _list);
            } else {
                if ext == "" || *ext == path.extension().unwrap().to_os_string().into_string().unwrap() {
                    let p: String = path.into_os_string().into_string().unwrap();
                    let md = std::fs::metadata(Path::new(&p)).unwrap();
                    let t: SystemTime = md.created().unwrap();

                    let fl = Files {
                        name: p,
                        create_date: t,
                        size: md.size() as usize,
                    };
                    _list.push(fl);
                }
            }
        }
    }
}


fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut ssize: &str;
    let mut log_write_flag = false;
    let mut log: String = String::from("");
    let mut ext: String = String::from("");
    let mut size: usize = 0;
    let mut log_file: FileOrNone = FileOrNone { f: None };
    let mut dry_run: bool = false;

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
                // show help
                println!("Command line arguments:");
                println!("  help - show this help");
                println!("  size=number - if number is greater then free space, the program will search for and deleting the oldest files. This parameter is required.");
                println!("  ext=extension - if specify this parameter, the application will only look for files with that extension.");
                println!("  log=log_file_name - if specify this parameter, the application will save the names of deleted files to this log file.");
                println!("  dry - process without real files deleting");
                return;
            },
            "dry" => {
                dry_run = true;
            },
            _ => {
                println!("Invalid parameter: {}",t_arg[0]);
                return;
            },
        }
    }

    if size == 0 {
        println!("You should specify the 'size' parameter in command line!");
        return;
    }

    // open log file
    if log_write_flag {
        log_file = match OpenOptions::new().read(true).write(true).create(true).open(log.clone()){
            Ok(l) => FileOrNone {f:Some(l)},
            Err(err) => {
                panic!("Error opening log file: {}", err.to_string());
            }
        };

        log_file.borrow_mut().unwrap().seek(SeekFrom::End(0)).expect("Error working with log file");

        log_file.write_log_message("Application started");
    }

    // analyze space size
    let cur_dir = env::current_dir().unwrap();
    let cwd = String::from(cur_dir.to_str().unwrap());
    let path_cstring =  CString::new(cwd.clone()).unwrap();
    let free_space: usize;
    unsafe {
        let mut stat_buf: libc::statvfs = mem::zeroed();
        libc::statvfs(path_cstring.as_ptr() as *const _, &mut stat_buf);
        free_space = stat_buf.f_bavail as usize * stat_buf.f_frsize as usize;
    }

    println!("Start point: {}", cwd);
    if log_write_flag {
        log_file.write_log_message(format!("Start point: {}", cwd).as_str());
    }
    println!("free space: {} bytes", free_space);

    if free_space < size {
        let mut list: Vec<Files> = Vec::new();
        // seek files to delete
        get_files_list(&cur_dir, &ext, &mut list);

        // sort and find slice of files to remove
        list.sort_by(|a,b| a.create_date.cmp(&b.create_date));
        let mut sum: usize = 0;
        let mut counter: u32 = 0;
        let files_number = list.len();
        if dry_run && log_write_flag {
            log_file.write_log_message("DRY mode");
        }
        for item in list {
            let res: Result<(), std::io::Error>;
            if dry_run {
                res = Ok(());
            } else {
                res = std::fs::remove_file(item.name.clone());
            }
            match res {
                Ok(_r) => {
                    sum += item.size;
                    counter += 1;

                    if log_write_flag {
                        log_file.write_log_message(format!("Cleared {} bytes by deleting {} ", item.size, item.name ).as_str());
                    }

                    if sum > size - free_space {
                        break;
                    }
                },
                Err(e) => {
                    println!("Something goes wrong during deleting file. {:?}", e);
                }
            }

        }
        println!("Checked {} files. deleted {} files. size of deleted files {} bytes.",files_number, counter, sum);

    }
    // finish

    if log_write_flag {
        log_file.write_log_message("Application finished");
    }
}
