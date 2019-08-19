{ pkgs ? import <nixpkgs> {} }:

let
  gdb-init = pkgs.writers.writeBashBin "gdb" ''
    ${pkgs.gdb}/bin/gdb "$@" \
      -ex "set tdesc filename ${./target-desc.xml}" \
      -ex "set debug remote 1"
  '';
in pkgs.mkShell {
  nativeBuildInputs = [ gdb-init ];
}
