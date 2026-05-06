let
  self = import ./. { };
  inherit (self) sources pkgs meta;

  cargoDependencySetOfCrate =
    crate:
    [ crate ]
    ++ pkgs.lib.concatMap cargoDependencySetOfCrate (crate.dependencies ++ crate.buildDependencies);
  cargoDependencySet = pkgs.lib.unique (
    pkgs.lib.flatten (
      pkgs.lib.mapAttrsToList (
        crateName: crate: cargoDependencySetOfCrate crate.build
      ) self.cargo.workspaceMembers
    )
  );
in
pkgs.mkShell rec {
  name = meta.operator.name;

  packages = with pkgs; [
    ## cargo et-al
    rustup # this breaks pkg-config if it is in the nativeBuildInputs

    ## Extra dependencies for use in a pure env (nix-shell --pure)
    ## These are mosuly useful for maintainers of this shell.nix
    ## to ensure all the dependencies are caught.
    # cacert
    # vim nvim nano
    crate2nix
  ];

  # derivation runtime dependencies
  buildInputs = pkgs.lib.concatMap (crate: crate.buildInputs) cargoDependencySet;

  # build time dependencies
  nativeBuildInputs =
    pkgs.lib.concatMap (crate: crate.nativeBuildInputs) cargoDependencySet
    ++ (with pkgs; [
      clang
      git
      # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
      llvmPackages.bintools
      rustup

      # additions for this repo
      go
    ]);

  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.glibc.dev}/include -I${pkgs.clang}/resource-root/include";

}
