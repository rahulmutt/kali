import Lake
open Lake DSL

package KaliProofs where
  srcDir := "."

@[default_target]
lean_lib KaliCore where
  roots := #[`KaliCore]

lean_lib KaliIR where
  roots := #[`KaliIR]
