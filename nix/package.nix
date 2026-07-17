{
  lib,
  rustPlatform,
  pkg-config,
}:
let
  cargoTOML = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "douglang";
  version = cargoTOML.package.version;

  src =
    let
      fs = lib.fileset;
      s = ../.;
    in
    fs.toSource {
      root = s;
      fileset = fs.unions [
        (s + /src)
        (s + /Cargo.lock)
        (s + /Cargo.toml)
        (s + /build.rs)
      ];
    };

  cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
  enableParallelBuilding = true;

  strictDeps = true;
  nativeBuildInputs = [
    pkg-config
  ];

  meta = {
    description = "Interpreter for Douglang esolang";
    license = lib.licenses.gpl3;
    maintainers = with lib.maintainers; [ Matercan ];
    mainProgram = "douglang";
    platforms = lib.platforms.linux;
  };
})
