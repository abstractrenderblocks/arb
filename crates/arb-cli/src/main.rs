fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Accept common version flags.
    if args.len() >= 2 {
        let a = args[1].as_str();
        if a == "--version" || a == "-V" || a == "version" {
            // Cargo sets this from [package].version
            let v = env!("CARGO_PKG_VERSION");
            println!("arb {}", v);
            return;
        }
    }

    // Minimal placeholder help (we'll replace once we implement real CLI parsing).
    eprintln!("arb {} (pre-alpha)", env!("CARGO_PKG_VERSION"));
    eprintln!("Usage:");
    eprintln!("  arb --version");
    eprintln!("  arb validate --package <name-or-path> --data <file>");
    eprintln!("  arb compile  --package <name-or-path> --data <file> --out <dir>");
    eprintln!("  arb init     --package <name-or-path> --out <file>");
    std::process::exit(2);
}
