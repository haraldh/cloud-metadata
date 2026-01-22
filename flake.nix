{
  description = "Fetch custom instance metadata from AWS, GCP, and Azure VMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    let
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      # Overlay that adds cloud-metadata to pkgs
      overlay = final: prev: {
        cloud-metadata = final.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          src = self;

          cargoLock.lockFile = ./Cargo.lock;

          meta = with final.lib; {
            description = "Fetch custom instance metadata from AWS, GCP, and Azure VMs";
            homepage = "https://github.com/haraldh/cloud-metadata";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
            mainProgram = "cloud-metadata";
          };
        };
      };
    in
    {
      # Overlay for other flakes to use
      overlays.default = overlay;

    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) overlay ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        packages = {
          cloud-metadata = pkgs.cloud-metadata;
          default = pkgs.cloud-metadata;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
          ];

          RUST_BACKTRACE = 1;
        };
      }
    );
}
