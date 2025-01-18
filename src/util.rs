use std::{fs, io, io::stdin};

pub fn query_directory(dir: &str) -> impl Iterator<Item = String> {
    let mut entries = fs::read_dir(dir).unwrap()
        .map(|res| res.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, io::Error>>().unwrap();
        entries.retain(|e| !e.eq_ignore_ascii_case(".DS_store"));

        entries.into_iter().map(|e| e.to_string_lossy().to_string())
}

pub fn read_stdin_bool() -> bool {
    let mut inp = String::new();
    loop {
        stdin().read_line(&mut inp).expect("Failed to read stdin");

        match inp.to_lowercase().trim() {
            "y" | "yes" | "true" | "t" | "1"  => {
                return true;
            },
            "n" | "no" | "false" | "f" | "0" => {
                return false;
            }
            _ => {
                println!("Invalid Input! Please only enter yes or no (or other aliases like y/n, t/f, etc)")
            }
        }
        inp.clear();
    }
}

pub fn read_stdin_u32() -> u32 {
    let mut inp = String::new();
    loop {
        stdin().read_line(&mut inp).expect("Failed to read stdin");

        if let Ok(num) = inp.trim().parse::<u32>() {
            return num;
        } else {
            println!("Invalid input! Please only enter valid positive integers");
        }
        inp.clear();
    }
}

pub fn read_stdin_usize() -> usize {
    let mut inp = String::new();
    loop {
        stdin().read_line(&mut inp).expect("Failed to read stdin");

        if let Ok(num) = inp.trim().parse::<usize>() {
            return num;
        } else {
            println!("Invalid input! Please only enter valid positive integers");
        }
        inp.clear();
    }
}

pub fn read_stdin_f32() -> f32 {
    let mut inp = String::new();
    loop {
        stdin().read_line(&mut inp).expect("Failed to read stdin");

        if let Ok(num) = inp.trim().parse::<f32>() {
            return num;
        } else {
            println!("Invalid input! Please only enter valid positive integers");
        }
        inp.clear();
    }
}
