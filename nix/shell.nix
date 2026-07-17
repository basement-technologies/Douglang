{
  mkShell,
  rustc,
  cargo,
  rust-analyzer,
  rustfmt,
  clippy,
  pkg-config,
  gcc,
  libclang,
}:

mkShell {
  name = "douglang-dev";
  strictDeps = true;

  nativeBuildInputs = [
    libclang
    cargo
    rustc
    clippy
    rustfmt
    rust-analyzer
    pkg-config
    gcc
  ];
}
