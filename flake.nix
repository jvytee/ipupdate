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
      crossTargets = [
        "aarch64-unknown-linux-musl"
        "x86_64-unknown-linux-musl"
      ];
    in {
      devShell = 
        let
          extraRustStds = system: targets: map (target: fenix.packages.${system}.targets.${target}.stable.rust-std) targets;
          toolchain = system: targets:
            with fenix.packages.${system}; combine ([
              stable.toolchain
            ] ++ (extraRustStds system targets));
        in 
          eachSystem (system:
            with import nixpkgs { inherit system; }; mkShell rec {
              nativeBuildInputs = [
                (toolchain system crossTargets)
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

          ipupd-aarch64 =
            with import nixpkgs { localSystem = system; crossSystem = { system = "aarch64-unknown-linux-musl"; rust.rustcTarget = "aarch64-unknown-linux-musl"; isStatic = true; }; };
            let
              toolchain =
                with fenix.packages.${system};
                combine [ stable.cargo stable.rustc targets.aarch64-unknown-linux-musl.stable.rust-std ];
              rustPlatform = makeRustPlatform { cargo = toolchain; rustc = toolchain; };
            in
              rustPlatform.buildRustPackage {
                pname = "ipupd-aarch64";
                version = "0.3.0";
                src = self;
                cargoLock.lockFile = ./Cargo.lock;

                # nativeBuildInputs = [ pkgsStatic.stdenv.cc ];
                env =
                  let 
                    cc = pkgsStatic.stdenv.cc;
                    ccPath = "${cc}/bin/${cc.targetPrefix}cc";
                  in {
                    # CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER = ccPath;
                    # CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
                    TARGET_CC = ccPath;
                  };
              };
        }
      );
    };
}
