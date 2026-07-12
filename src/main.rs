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
use std::process;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: douglang <input.doug> [--compile [out.c]]");
        process::exit(1);
    }

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

    if comp_mode {
        let out = args
            .iter()
            .position(|a| a == "--compile")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("out.c");

        let mut comp = compiler::Compiler::new();
        let c = comp.compile(&ast);
        let elapsed = start.elapsed();

        if let Err(e) = fs::write(out, &c) {
            eprintln!("couldn't write output file: {e}");
            process::exit(1);
        }

        println!(
            "compiled to {out} in {:.6} seconds",
            elapsed.as_secs_f64()
        );
    } else {
        let elapsed = start.elapsed();
        eprintln!("parsed in {:.6} seconds", elapsed.as_secs_f64());

        let tts = Arc::new(tts::Tts::new());

        let mut gui = dougterface::Dougterface::new(&tts);
        gui.start(&tts);

        let mut interp = interpreter::Interpreter::new(Arc::clone(&tts));
        if let Err(e) = interp.run(&ast) {
            eprintln!("{}", e.message);
            process::exit(1);
        }

        tts.wait();
        gui.stop();
    }
}
