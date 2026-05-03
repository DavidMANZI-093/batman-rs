#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("<3>[batman ERROR] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("<4>[batman WARN] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("<6>[batman INFO] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("<7>[batman DEBUG] {}", format_args!($($arg)*));
    };
}
