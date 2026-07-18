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
  speechd,
  glib,
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

  buildInputs = [
    speechd
    glib
  ];

  shellHook = ''
    export LIBCLANG_PATH="${libclang.lib}/lib"
    export BINDGEN_EXTRA_CLANG_ARGS="-I${speechd}/include"
  '';
}
