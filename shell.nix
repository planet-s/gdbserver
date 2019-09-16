{ pkgs ? import <nixpkgs> {} }:

let
  gdb-init = pkgs.writers.writeBashBin "gdb" ''
    ${pkgs.gdb}/bin/gdb \
      -ex "set tdesc filename ${./target-desc.xml}" \
      "$@"
  '';
  gdb-test = pkgs.writers.writeBashBin "gdb-test" ''
    ${gdb-init}/bin/gdb \
      -ex "set debug remote 1" \
      -ex "target remote :64126" \
      "$@"
  '';
in pkgs.mkShell {
  nativeBuildInputs = [ gdb-init gdb-test pkgs.musl ];
}
