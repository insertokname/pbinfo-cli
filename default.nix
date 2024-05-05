{pkgs?import <nixpkgs>{}}:
pkgs.mkShell {
  buildInputs = with pkgs; [ cargo openssl pkg-config ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
