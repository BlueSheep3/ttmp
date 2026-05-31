# Unlike the rest of ttmp, this flake is not licensed under GPL-3.0-or-later.
# This flake is unlicensed, you can do whatever you want with it.
#
# This flake is heavily based on:
# https://kampffrosch94.github.io/posts/nix_win_cross/
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    name = "ttmp";
    version = "0.3.2";
    pkgs = import nixpkgs {inherit system;};

    rpathLibs = with pkgs; [
      alsa-lib
      dbus
    ];
    ttmp = pkgs.rustPlatform.buildRustPackage {
      inherit name version;
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = rpathLibs;
      postFixup = ''patchelf --add-rpath "${pkgs.lib.makeLibraryPath rpathLibs}" $out/bin/${name}'';

      postInstall = ''
        mkdir -p "$out/share/icons/hicolor/512x512/apps"
        cp 'assets/icon.png' "$out/share/icons/hicolor/512x512/apps/${name}.png"

        mkdir -p "$out/share/applications"
        cp 'assets/desktopfile.desktop' "$out/share/applications/${name}.desktop"
      '';
    };

    pkgs-windows = pkgs.pkgsCross.mingwW64;
    ttmp-windows = pkgs-windows.rustPlatform.buildRustPackage {
      inherit name version;
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
    };
  in {
    packages.${system} = {
      default = ttmp;
      inherit ttmp ttmp-windows;
    };
  };
}
