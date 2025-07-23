# /path/to/your/apps/ratatat-rust/flake.nix
{
  description = "A NixOS daemon to listen for the 'ratatat' key sequence.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      # This is now a Home Manager module.
      homeManagerModules.default = { config, lib, ... }:
        let
          # Define the package that this module provides.
          ratatat-pkg = pkgs.rustPlatform.buildRustPackage rec {
            pname = "ratatat-listener";
            version = "0.1.0";
            src = self;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = with pkgs.xorg; [
              pkgs.udev
              pkgs.libinput
              libX11
              libXtst
              libXi
              xorgproto
            ];

            # This hook runs AFTER the default install phase, which correctly
            # places the binary in $out/bin. We then add our sound file.
            postInstall = ''
              mkdir -p $out/share
              cp ${src}/Loud-pipes.mp3 $out/share/
            '';
          };
        in
        {
          # This creates the option you will use in your home.nix
          options.services.ratatat-listener.enable = lib.mkEnableOption "Enable the ratatat listener user service";

          config = lib.mkIf config.services.ratatat-listener.enable {
            # Install the package into the user's profile.
            home.packages = [
              ratatat-pkg
              pkgs.mpg123
            ];

            # Define the systemd USER service.
            systemd.user.services.ratatat-listener = {
              Unit = {
                Description = "Listens for the 'ratatat' key sequence and plays a song.";
                After = [ "graphical-session.target" ];
              };
              Install = {
                WantedBy = [ "graphical-session.target" ];
              };
              Service = {
                # This sets the environment variable for the Rust program to find the song.
                Environment = "RATATAT_SONG_PATH=${ratatat-pkg}/share/Loud-pipes.mp3";
                ExecStart = "${ratatat-pkg}/bin/ratatat-listener";
                Restart = "always";
                RestartSec = "5s";
              };
            };
          };
        };
    };
}

