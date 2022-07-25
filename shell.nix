with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "dyns";
    nativeBuildInputs = [ pkg-config zlib openssl ];
    buildInputs = [ cargo rustc ];
}
