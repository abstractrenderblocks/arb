use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Version flags
    if args.len() >= 2 {
        let a = args[1].as_str();
        if a == "--version" || a == "-V" || a == "version" {
            println!("arb {}", env!("CARGO_PKG_VERSION"));
            return;
        }
    }

    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let cmd = args[1].as_str();
    match cmd {
        "validate" => cmd_validate(&args[2..]),
        "compile" | "init" => {
            eprintln!("arb {} (pre-alpha)", env!("CARGO_PKG_VERSION"));
            eprintln!("Command not implemented yet: {cmd}");
            std::process::exit(2);
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn cmd_validate(args: &[String]) {
    // Minimal flag parser:
    // arb validate --package <name-or-path> --data <file>
    let mut pkg: Option<String> = None;
    let mut data: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--package" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value after --package");
                    print_validate_usage();
                    std::process::exit(2);
                }
                pkg = Some(args[i].clone());
            }
            "--data" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value after --data");
                    print_validate_usage();
                    std::process::exit(2);
                }
                data = Some(PathBuf::from(&args[i]));
            }
            "--help" | "-h" => {
                print_validate_usage();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {a}");
                print_validate_usage();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let pkg = match pkg {
        Some(p) => p,
        None => {
            eprintln!("Missing required option: --package");
            print_validate_usage();
            std::process::exit(2);
        }
    };

    let data = match data {
        Some(d) => d,
        None => {
            eprintln!("Missing required option: --data");
            print_validate_usage();
            std::process::exit(2);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    match arb_core::validate_command(&cwd, &pkg, Path::new(&data)) {
        Ok(errs) => {
            if errs.is_empty() {
                println!("OK");
                std::process::exit(0);
            }

            eprintln!("Schema validation failed ({} error(s)):", errs.len());
            for e in errs {
                eprintln!("  {}: {}", e.path, e.message);
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("arb {} (pre-alpha)", env!("CARGO_PKG_VERSION"));
    eprintln!("Usage:");
    eprintln!("  arb --version");
    eprintln!("  arb validate --package <name-or-path> --data <file>");
}

fn print_validate_usage() {
    eprintln!("Usage:");
    eprintln!("  arb validate --package <name-or-path> --data <file>");
}
