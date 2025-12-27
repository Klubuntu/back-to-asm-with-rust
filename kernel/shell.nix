{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    nasm
    binutils
    qemu
  ];
  
  shellHook = ''
    echo "Rust development shell for Multiboot kernel"
    echo ""
    echo "First time setup:"
    echo "  rustup default nightly"
    echo "  rustup component add rust-src"
    echo ""
    echo "Then run: ./make_multiboot.sh"
  '';
}
