use clap::{ArgAction, Parser};
use douglang::values::tape::Memory;
use douglang::*;

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::time::Instant;

/// The Doug language interpreter / compiler.
///
/// Normal usage: `douglang <input.doug> [--compile [output.c]] [--cc [binary]]
/// [--link <lib>...] [--no-gui]`
///
/// The `--run-source-helper`, `--tts-helper`, `--tts-helper-quiet`, and
/// `--dougterface-helper` flags are internal re-exec entry points used by the
/// interpreter itself (via `env::current_exe()`), not meant for direct use.
#[derive(Parser, Debug)]
#[command(name = "douglang")]
struct Cli {
	/// Input .doug source file (normal invocation)
	input: Option<String>,
	/// Compile to C source instead of interpreting. Optional output path
	/// (defaults to `<input>.c`).
	#[arg(long, short = 'c', num_args = 0..=1, default_missing_value = "")]
	compile: Option<String>,
	/// Compile and link with gcc. Optional binary name (defaults to
	/// `<input>.out` / `<input>.exe`).
	#[arg(long, num_args = 0..=1, default_missing_value = "")]
	cc: Option<String>,
	/// Link additional libraries. Repeatable (`--link a --link b`) or
	#[arg(long = "link", short = 'l', num_args = 1.., action = ArgAction::Append)]
	link: Vec<String>,
	#[arg(long = "no-gui")]
	no_gui: bool,
	#[arg(long = "pure", short = 'p')]
	/// Run without executing any overhead (tts prints to stdout and there is no gui)
	pure: bool,
	// --- internal helper entry points (hidden from --help) ---
	#[arg(long = "run-source-helper", hide = true, value_name = "PATH")]
	run_source_helper: Option<String>,
	#[arg(long = "tts-helper", hide = true, num_args = 2..=3, value_names = ["MODE", "PATH", "STATE"])]
	tts_helper: Option<Vec<String>>,
	#[arg(long = "tts-helper-quiet", hide = true, num_args = 2..=3, value_names = ["MODE", "PATH", "STATE"])]
	tts_helper_quiet: Option<Vec<String>>,
	#[arg(long = "dougterface-helper", hide = true, value_name = "PATH")]
	dougterface_helper: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let cli = Cli::parse();

	if let Some(path) = cli.run_source_helper {
		run_source_helper(&path, cli.link);
		return Ok(());
	}

	if let Some(vals) = cli.tts_helper {
		run_tts_helper(vals, false);
		return Ok(());
	}

	if let Some(vals) = cli.tts_helper_quiet {
		run_tts_helper(vals, true);
		return Ok(());
	}

	if let Some(path) = cli.dougterface_helper {
		dougterface::run_file_helper(Path::new(&path).to_path_buf());
		return Ok(());
	}

	let Some(input_path) = cli.input else {
		eprintln!(
			"usage: douglang <input.doug> [--compile [output.c]] [--cc [binary]] [--link <lib>...] [--no-gui]"
		);
		process::exit(1);
	};

	let input_name = Path::new(&input_path)
		.file_stem()
		.and_then(|s| s.to_str())
		.unwrap_or("out")
		.to_string();

	let source = match fs::read_to_string(&input_path) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("couldn't read input file: {e}");
			process::exit(1);
		}
	};

	let start = Instant::now();
	let mem = Memory::new();

	let linked_libs = cli.link;

	if cli.compile.is_some() || cli.cc.is_some() {
		let c_path = match &cli.compile {
			Some(p) if !p.is_empty() => p.clone(),
			_ => format!("{input_name}.c"),
		};

		let mut parser = douglang::parser::Parser::new(input_path);
		let ast = mem.mutate(&mut parser, ())?;

		let helper_path = env::current_exe()
			.ok()
			.and_then(|p| p.to_str().map(|s| s.to_string()));
		let mut comp = compiler::Compiler::new(helper_path, source.clone(), linked_libs.clone());
		let c_code = match comp.compile(&ast, &linked_libs) {
			Ok(c_code) => c_code,
			Err(e) => {
				eprintln!("{e}");
				process::exit(1);
			}
		};
		let elapsed = start.elapsed();

		if let Err(e) = fs::write(&c_path, &c_code) {
			eprintln!("couldn't write output file: {e}");
			process::exit(1);
		}

		println!(
			"compiled to {c_path} in {:.6} seconds",
			elapsed.as_secs_f64()
		);

		if let Some(binary_opt) = &cli.cc {
			let binary_name = if !binary_opt.is_empty() {
				binary_opt.clone()
			} else if cfg!(windows) {
				format!("{input_name}.exe")
			} else {
				format!("{input_name}.out")
			};

			let mut gcc_args: Vec<String> =
				vec!["-o".into(), binary_name.clone(), c_path.to_string()];

			for lib in &linked_libs {
				if lib.ends_with(".c") {
					continue;
				}
				let is_path = lib.contains('/')
					|| lib.contains('\\')
					|| lib.ends_with(".dll")
					|| lib.ends_with(".so")
					|| lib.ends_with(".a")
					|| lib.ends_with(".dylib");
				if is_path {
					gcc_args.push(lib.clone());
				} else {
					gcc_args.push(format!("-l{lib}"));
				}
			}

			gcc_args.extend(resolve_pkg_config(&linked_libs));

			let status = process::Command::new("gcc").args(&gcc_args).status();

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

		if cli.pure {
			let mut interp = interpreter::Interpreter::new(None, linked_libs, input_path);

			if let Err(e) = mem.mutate(&mut interp, ()) {
				eprintln!("{e}");
				process::exit(1);
			}
		} else {
			let tts = Arc::new(douglang::tts::Tts::new());

			let mut gui = dougterface::Dougterface::new(&tts);
			if !cli.no_gui {
				gui.start(&tts);
			}

			let mut interp =
				interpreter::Interpreter::new(Some(Arc::clone(&tts)), linked_libs, input_path);

			if let Err(e) = mem.mutate(&mut interp, ()) {
				eprintln!("{e}");
				process::exit(1);
			}

			tts.wait();
			gui.stop();
		}
	}

	Ok(())
}

fn run_source_helper(path: &str, linked_libs: Vec<String>) {
	let tts = Arc::new(douglang::tts::Tts::new());
	let mut gui = dougterface::Dougterface::new(&tts);
	gui.start(&tts);
	let mut interpreter = interpreter::Interpreter::new(Some(Arc::clone(&tts)), linked_libs, path);
	let memory = Memory::new();

	if let Err(e) = memory.mutate(&mut interpreter, ()) {
		eprintln!("{e}");
		process::exit(1);
	}
	tts.wait();
	gui.stop();
}

fn run_tts_helper(vals: Vec<String>, quiet: bool) {
	// vals is [mode, path] or [mode, path, state_path], enforced by num_args(2..=3)
	let mode = &vals[0];
	let path = &vals[1];
	let state_path = vals.get(2).cloned();

	let text = match fs::read_to_string(path) {
		Ok(text) => text,
		Err(e) => {
			eprintln!("couldn't read tts helper text: {e}");
			process::exit(1);
		}
	};
	let _ = fs::remove_file(path);
	let tts = douglang::tts::Tts::new();
	if let Some(state_path) = state_path {
		tts.set_state_file(state_path);
	}
	if quiet {
		tts.speak_audio_only(&text, mode == "overlap");
	} else if mode == "overlap" {
		tts.speak_overlap(&text);
		tts.wait();
	} else {
		tts.speak(&text);
	}
}

fn resolve_pkg_config(libs: &[String]) -> Vec<String> {
	let mut flags = Vec::new();
	for lib in libs {
		if let Ok(output) = process::Command::new("pkg-config")
			.args(["--libs", lib])
			.output() && output.status.success()
			&& let Ok(out) = String::from_utf8(output.stdout)
		{
			for flag in out.split_whitespace() {
				flags.push(flag.to_string());
			}
		}
	}
	flags
}
