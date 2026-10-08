{ pkgs, lib, inputs, ... }:
let
  inherit (pkgs.stdenv.hostPlatform) isDarwin isLinux;

  rustBin = inputs.rust-overlay.lib.mkRustBin { } pkgs.buildPackages;
  toolchainFrom = file: rustBin.fromRustupToolchainFile file;

  # rustfmt runs on the nightly in tools/rustfmt for its unstable options. Only rustfmt and
  # cargo-fmt are exposed, so that toolchain's cargo and rustc can't shadow the pinned ones.
  rustfmt = pkgs.runCommand "rustfmt-nightly" { } ''
    mkdir -p $out/bin
    ln -s ${toolchainFrom ./tools/rustfmt/rust-toolchain.toml}/bin/{rustfmt,cargo-fmt} $out/bin/
  '';

  # Hawk uses rustc_private, so it only runs on the exact toolchain it was built against,
  # pinned in tools/hawk. The wrapper hands it that toolchain whichever cargo invokes it.
  hawkToolchain = toolchainFrom ./tools/hawk/rust-toolchain.toml;
  # cargo-hawk points the dynamic loader at the toolchain's libraries so its driver can find
  # librustc_driver; the driver gets an rpath below instead. Left set, the variable reaches
  # Nix's clang when cargo links build scripts and loads rustc's LLVM into it. (Apple's cc
  # is SIP-protected and never sees it, which is why rustup setups don't hit this.)
  hawkCargo = pkgs.writeShellScriptBin "cargo" ''
    unset DYLD_LIBRARY_PATH LD_LIBRARY_PATH
    exec ${hawkToolchain}/bin/cargo "$@"
  '';
  hawk = pkgs.stdenv.mkDerivation (finalAttrs: {
    pname = "cargo-hawk";
    version = "0.1.14";
    src = pkgs.fetchurl (
      let
        release = target: hash: {
          url = "https://github.com/astral-sh/hawk/releases/download/${finalAttrs.version}/cargo-hawk-${target}.tar.gz";
          inherit hash;
        };
      in
      {
        aarch64-darwin = release "aarch64-apple-darwin" "sha256-oTeWICRbpW1TvKNyf1ex25WbYIW0pk3EKlbxI2f9ZMY=";
        x86_64-linux = release "x86_64-unknown-linux-gnu" "sha256-bMi1fg2R7b33A4glTHPi3PdlOSnYMIHkfcKm1shbA2k=";
        aarch64-linux = release "aarch64-unknown-linux-gnu" "sha256-KjbtYkSntkatoMfpvJDLwXV/F8xK9mdeyzLuwp193iM=";
      }.${pkgs.stdenv.hostPlatform.system}
    );
    nativeBuildInputs = [ pkgs.makeWrapper ] ++ lib.optional isLinux pkgs.autoPatchelfHook;
    buildInputs = lib.optionals isLinux [ hawkToolchain pkgs.stdenv.cc.cc.lib ];
    installPhase = ''
      install -Dm755 -t $out/bin cargo-hawk cargo-hawk-driver
    '' + lib.optionalString isDarwin ''
      install_name_tool -add_rpath ${hawkToolchain}/lib $out/bin/cargo-hawk-driver
    '' + ''
      wrapProgram $out/bin/cargo-hawk \
        --prefix PATH : ${hawkCargo}/bin:${hawkToolchain}/bin \
        --set CARGO ${hawkCargo}/bin/cargo \
        --set RUSTC ${hawkToolchain}/bin/rustc
    '';
    dontStrip = true;
  });

  # GPUI's native dependencies on Linux; it dlopens the display and GPU libraries at runtime.
  linuxRuntimeLibs = with pkgs; [
    fontconfig
    freetype
    libxkbcommon
    vulkan-loader
    wayland
    xorg.libX11
    xorg.libxcb
  ];
in
{
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
    lsp.package = rustBin.stable.${(lib.importTOML ./rust-toolchain.toml).toolchain.channel}.rust-analyzer;
  };

  packages = [
    hawk
    rustfmt
    pkgs.just
    pkgs.prek
    pkgs.zizmor
  ] ++ lib.optionals isLinux ([ pkgs.cmake pkgs.pkg-config ] ++ linuxRuntimeLibs);

  # `devenv test`: everything CI checks
  enterTest = ''
    just fmt --check
    just clippy
    just hawk
    just test
    prek run --all-files --skip cargo-fmt --skip cargo-clippy
  '';

  env = {
    # editors' format-on-save goes through rust-analyzer, which honors this too
    RUSTFMT = "${rustfmt}/bin/rustfmt";
  } // lib.optionalAttrs isLinux {
    LD_LIBRARY_PATH = lib.makeLibraryPath linuxRuntimeLibs;
  };
}
