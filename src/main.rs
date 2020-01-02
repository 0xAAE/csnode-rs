mod config;

fn main() {
    println!("Hello, world!");

    let mut file_name = "config.ini".to_string();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("--config=") {
            match arg.split("=").skip(1).next() {
                Some(v) => {
                    file_name = v.to_string();
                }
                None => {
                    println!("Incorrect option value for --config, must be --config=<file name>");
                }
            };
        }
    }
    let config = config::Config::new(&file_name);
}
