{ pkgs ? import <nixpkgs> {} }:

let
  gdb-test = pkgs.writers.writeBashBin "gdb-test" ''
    ${gdb-init}/bin/gdb \
      -ex "set debug remote 1" \
      -ex "target remote :64126" \
      "$@"
  '';
in pkgs.mkShell {
  nativeBuildInputs = [ gdb-test ];
}
