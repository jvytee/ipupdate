{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      eachSystem = nixpkgs.lib.genAttrs systems;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in {
      devShells =
        eachSystem (system:
          {
            default = 
              with import nixpkgs { inherit system; };
              mkShell {
                nativeBuildInputs = [
                  cargo
                  clippy
                  gdb
                  gh
                  openssl
                  pkg-config
                  rust-analyzer
                  rustc
                  rustfmt
                  yaml-language-server
                ];

                CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER =
                  let cc = pkgsStatic.stdenv.cc;
                  in "${cc}/bin/${cc.targetPrefix}cc";
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER =
                  let cc = pkgsCross.aarch64-multiplatform-musl.pkgsStatic.stdenv.cc;
                  in "${cc}/bin/${cc.targetPrefix}cc";
                # CARGO_BUILD_RUSTFLAGS = [ "-C" "target-feature=+crt-static" ];
                # TARGET_CC = "${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER}";
              };

            test =
              with import nixpkgs { inherit system; };
              mkShell {
                nativeBuildInputs = [
                  cargo
                  openssl
                  pkg-config
                  rustc
                ];
              };
          }
        );

      packages = eachSystem (system:
        {
          default =
            with import nixpkgs { inherit system; };
            rustPlatform.buildRustPackage {
              pname = "ipupdate";
              version = "0.3.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
            };

          ipupdate-x86_64 =
            with import nixpkgs {
              localSystem = system;
              crossSystem = {
                system = "x86_64-unknown-linux-musl";
                isStatic = true;
                rustc.rustcTarget = "x86_64-unknown-linux-musl";
              };
            };
            rustPlatform.buildRustPackage {
              pname = "ipupdate";
              version = "0.3.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
            };

          ipupdate-aarch64 =
            with import nixpkgs {
              localSystem = system;
              crossSystem = {
                system = "aarch64-unknown-linux-musl";
                isStatic = true;
                rustc.rustcTarget = "aarch64-unknown-linux-musl";
              };
            };
            rustPlatform.buildRustPackage {
              pname = "ipupdate";
              version = "0.3.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
            };
        }
      );
    };
}
