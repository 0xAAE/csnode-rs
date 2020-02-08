fn main() {
    println!(r"cargo:rustc-link-search=native=./third-party/p2p_compat/build");
    println!(r"cargo:rustc-link-lib=p2p_compat");
    println!(r"cargo:rustc-link-lib=p2p_network");
    println!(r"cargo:rustc-link-lib=bitcoin_utils");
    println!(r"cargo:rustc-link-lib=easylogging");
    println!(r"cargo:rustc-link-lib=miniupnpc");
    println!("cargo:rustc-link-lib=static=stdc++");
}