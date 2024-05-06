{pkgs?import <nixpkgs>{}}:
pkgs.mkShell {
  buildInputs = with pkgs; [ cargo rustc openssl pkg-config rustfmt rust-analyzer];
}
