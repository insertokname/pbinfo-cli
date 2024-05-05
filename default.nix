let
  pkgs = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-23.11.tar.gz";
    # user `nix-prefetch-url --unpack` to ge sha 
    sha256 = "00rghzfjah557h1f3nynnbxlb3l967p0m1krmqagfs4a9cq39f5m";
  }) {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [ cargo openssl pkg-config ];
  }
