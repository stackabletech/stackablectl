{
  sources ? import ./nix/sources.nix, # managed by https://github.com/nmattia/niv
  nixpkgs ? sources.nixpkgs,
  pkgs ? import nixpkgs {
    overlays = [
      # gomod2nix must be imported as a nixpkgs overlay
      (import (sources.gomod2nix + "/overlay.nix"))
    ];
  },
  release ? true,
  cargo ? import ./Cargo.nix {
    inherit nixpkgs pkgs release;
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      helm-sys = attrs: {
        GO_HELM_WRAPPER = goHelmWrapper + "/bin";
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.glibc.dev}/include -I${pkgs.clang.cc.lib}/lib/clang/${pkgs.lib.versions.major (pkgs.lib.getVersion pkgs.clang.cc)}/include";
      };
      utoipa-swagger-ui = attrs: rec {
        # utoipa-swagger-ui tries to redownload swagger-ui, which is blocked by Nix's sandboxing
        # so we download it instead, and tell it to use that
        SWAGGER_UI_DOWNLOAD_URL = "file://${
          pkgs.fetchurl {
            url = "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.3.zip";
            hash = "sha256-zrb8feuuDzt/g6y7Tucfh+Y2BWZov0soyNPR5LBqKx4=";
          }
        }";
      };
    };
  },
  meta ? pkgs.lib.importJSON ./nix/meta.json,
  dockerName ? "oci.stackable.tech/sandbox/${meta.operator.name}",
  dockerTag ? null,
  goHelmWrapper ? pkgs.buildGoApplication {
    go = pkgs.go_1_26;
    pname = "go-helm-wrapper";
    version = "0.0";
    src = pkgs.runCommand "go-helm-wrapper-src" { } ''
      mkdir $out
      cp ${./go.mod} $out/go.mod
      cp ${./go.sum} $out/go.sum
      cp -r ${./rust/helm-sys/go-helm-wrapper} $out/go-helm-wrapper
    '';
    pwd = ./rust/helm-sys/go-helm-wrapper;
    modules = ./gomod2nix.toml;
    ldflags = "-buildmode c-archive";
    allowGoReference = true;
    postBuild = ''
      for pkg in $(getGoDirs ""); do
        buildFlags="-buildmode c-archive -o $GOPATH/bin/libgo-helm-wrapper.a" buildGoDir build "$pkg"
      done
    '';
  },
  gomod2nix ? pkgs.callPackage sources.gomod2nix { },
}:
rec {
  inherit
    cargo
    sources
    pkgs
    meta
    ;
  build = cargo.workspaceMembers.stackable-cockpitd.build;
  entrypoint = build + "/bin/stackable-cockpitd";
  # crds = pkgs.runCommand "${meta.operator.name}-crds.yaml" {}
  # ''
  #   ${entrypoint} crd > $out
  # '';

  dockerImage = pkgs.dockerTools.streamLayeredImage {
    name = dockerName;
    tag = dockerTag;
    contents = [
      # Common debugging tools
      pkgs.bashInteractive
      pkgs.coreutils
      pkgs.util-linuxMinimal
      # Kerberos 5 must be installed globally to load plugins correctly
      pkgs.krb5
      # Make the whole cargo workspace available on $PATH
      build
    ];
    config = {
      Env =
        let
          fileRefVars = {
            PRODUCT_CONFIG = deploy/config-spec/properties.yaml;
          };
        in
        pkgs.lib.concatLists (
          pkgs.lib.mapAttrsToList (
            env: path: pkgs.lib.optional (pkgs.lib.pathExists path) "${env}=${path}"
          ) fileRefVars
        );
      Entrypoint = [ entrypoint ];
      Cmd = [ ];
    };
  };
  docker = pkgs.linkFarm "stackable-cockpit-docker" [
    {
      name = "load-image";
      path = dockerImage;
    }
    {
      name = "ref";
      path = pkgs.writeText "${dockerImage.name}-image-tag" "${dockerImage.imageName}:${dockerImage.imageTag}";
    }
    {
      name = "image-repo";
      path = pkgs.writeText "${dockerImage.name}-repo" dockerImage.imageName;
    }
    {
      name = "image-tag";
      path = pkgs.writeText "${dockerImage.name}-tag" dockerImage.imageTag;
    }
    # {
    #   name = "crds.yaml";
    #   path = crds;
    # }
  ];

  # need to use vendored crate2nix because of https://github.com/kolloch/crate2nix/issues/264
  crate2nix = import sources.crate2nix { };
  tilt = pkgs.tilt;

  regenerateNixLockfiles = pkgs.writeScriptBin "regenerate-nix-lockfiles" ''
    #!/usr/bin/env bash
    set -euo pipefail
    echo Running crate2nix
    ${crate2nix}/bin/crate2nix generate
    echo Running gomod2nix
    ${gomod2nix}/bin/gomod2nix
  '';
}
