{
	description = "Rust development shell";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay = {
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
		flake-utils.lib.eachDefaultSystem (system:
			let
				overlays = [ (import rust-overlay) ];
				pkgs = import nixpkgs { inherit system overlays; };
				rust = pkgs.rust-bin.nightly.latest.default.override {
					extensions = [ "miri" "rust-src" ];
				};
			in {
				devShells.default = pkgs.mkShell {
					packages = [ rust ];
				};
			});
}
