{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
    let
      eachSystem = nixpkgs.lib.genAttrs systems;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      toolchain = system: target: with fenix.packages.${system}; combine [
        stable.toolchain
        targets.${target}.stable.rust-std
      ];
    in
      {
      devShell = eachSystem (system:
        with import nixpkgs { inherit system; };
        mkShell rec {
          nativeBuildInputs = [
            (toolchain system "aarch64-unknown-linux-musl")
            yaml-language-server
            pkgsCross.aarch64-multiplatform-musl.pkgsStatic.stdenv.cc
          ];

          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER =
            let cc = pkgsCross.aarch64-multiplatform-musl.pkgsStatic.stdenv.cc;
            in "${cc}/bin/${cc.targetPrefix}cc";
          CARGO_BUILD_RUSTFLAGS = [ "-C" "target-feature=+crt-static" ];
          TARGET_CC = "${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER}";
        }
      );

      packages = eachSystem (system:
        {
          default =
            with import nixpkgs { inherit system; };
            let
              toolchain =
                with fenix.packages.${system};
                combine [ stable.cargo stable.rustc ];
              rustPlatform = makeRustPlatform { cargo = toolchain; rustc = toolchain; };
            in
              rustPlatform.buildRustPackage {
                pname = "ipupd";
                version = "0.3.0";
                src = self;
                cargoLock.lockFile = ./Cargo.lock;
              };
        }
      );
    };
}
