mod ast;
mod compiler;
mod dougterface;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod tts;

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: douglang <input.doug> [--compile [output.c]] [--cc] [--link <lib>...]");
        process::exit(1);
    }

    let input_name = Path::new(&args[1])
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out")
        .to_string();

    let source = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("couldn't read input file: {e}");
            process::exit(1);
        }
    };

    let start = Instant::now();

    let tokens = match lexer::lex(&source) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let ast = match parser::parse(&tokens) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let comp_mode = args.iter().any(|a| a == "--compile");
    let cc_mode = args.iter().any(|a| a == "--cc");

    let mut linked_libs: Vec<String> = Vec::new();
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--link" {
            i += 1;
            let mut found = false;
            while i < args.len() && !args[i].starts_with("--") {
                linked_libs.push(args[i].clone());
                i += 1;
                found = true;
            }
            if !found {
                eprintln!("--link requires a library name");
                process::exit(1);
            }
            continue;
        }
        i += 1;
    }

    if comp_mode || cc_mode {
        let c_path = args
            .iter()
            .position(|a| a == "--compile")
            .and_then(|i| args.get(i + 1))
            .filter(|s| !s.starts_with("--"))
            .cloned()
            .unwrap_or(format!("{input_name}.c"));

        let mut comp = compiler::Compiler::new();
        let c_code = comp.compile(&ast, &linked_libs);
        let elapsed = start.elapsed();

        if let Err(e) = fs::write(&c_path, &c_code) {
            eprintln!("couldn't write output file: {e}");
            process::exit(1);
        }

        println!(
            "compiled to {c_path} in {:.6} seconds",
            elapsed.as_secs_f64()
        );

        if cc_mode {
            let binary_name = args
                .iter()
                .position(|a| a == "--cc")
                .and_then(|i| args.get(i + 1))
                .filter(|s| !s.starts_with("--"))
                .cloned()
                .unwrap_or_else(|| {
                    if cfg!(windows) {
                        format!("{input_name}.exe")
                    } else {
                        format!("{input_name}.out")
                    }
                });

            let mut gcc_args: Vec<String> = vec![
                "-o".into(),
                binary_name.clone(),
                c_path.to_string(),
            ];

            for lib in &linked_libs {
                let is_path = lib.contains('/') || lib.contains('\\')
                    || lib.ends_with(".dll") || lib.ends_with(".so") || lib.ends_with(".a") || lib.ends_with(".dylib");
                if is_path {
                    gcc_args.push(lib.clone());
                } else {
                    gcc_args.push(format!("-l{lib}"));
                }
            }

            gcc_args.extend(resolve_pkg_config(&linked_libs));

            let status = process::Command::new("gcc")
                .args(&gcc_args)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("linked to {binary_name}");
                }
                Ok(s) => {
                    eprintln!("gcc exited with code {}", s.code().unwrap_or(-1));
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("couldn't run gcc: {e}");
                    process::exit(1);
                }
            }
        }
    } else {
        let elapsed = start.elapsed();
        eprintln!("parsed in {:.6} seconds", elapsed.as_secs_f64());

        let tts = Arc::new(tts::Tts::new());

        let mut gui = dougterface::Dougterface::new(&tts);
        gui.start(&tts);

        let mut interp = interpreter::Interpreter::new(Arc::clone(&tts), linked_libs);
        if let Err(e) = interp.run(&ast) {
            eprintln!("{}", e.message);
            process::exit(1);
        }

        tts.wait();
        gui.stop();
    }
}

fn resolve_pkg_config(libs: &[String]) -> Vec<String> {
    let mut flags = Vec::new();
    for lib in libs {
        if let Ok(output) = process::Command::new("pkg-config")
            .args(["--libs", lib])
            .output()
        {
            if output.status.success() {
                if let Ok(out) = String::from_utf8(output.stdout) {
                    for flag in out.split_whitespace() {
                        flags.push(flag.to_string());
                    }
                }
            }
        }
    }
    flags
}
