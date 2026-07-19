{
  lib,
  rustPlatform,
  pkg-config,
  speechd,
  makeWrapper,
  glib
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
      ];
    };

  cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
  enableParallelBuilding = true;

  strictDeps = true;
  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = [
    speechd
    glib
  ];

  postFixup = ''
    wrapProgram $out/bin/voxelmint \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [
          speechd
          glib
        ]
      }
  '';

  meta = {
    description = "Interpreter for Douglang esolang";
    license = lib.licenses.gpl3;
    maintainers = with lib.maintainers; [ Matercan ];
    mainProgram = "douglang";
    platforms = lib.platforms.linux;
  };
})
