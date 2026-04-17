{ pkgs, lib, config, inputs, ... }:

{
  packages = [ pkgs.lean4 ];

  enterShell = ''
    echo "Lean 4 environment ready: $(lean --version)"
    echo "Run proofs with: cd proofs && lake build"
  '';
}
