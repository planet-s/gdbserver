{ pkgs ? import <nixpkgs> {} }:

let
  gdb-test = pkgs.writers.writeBashBin "gdb-test" ''
    ${pkgs.gdb}/bin/gdb \
      -ex "set debug remote 1" \
      -ex "target remote :64126" \
      "$@"
  '';
in pkgs.mkShell {
  nativeBuildInputs = [ gdb-test ];
}
