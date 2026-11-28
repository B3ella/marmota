{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      mkMarmotaPackage = pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "marmota";
          version = "2.0.2";

          cargoLock.lockFile = ./Cargo.lock;
          src = self;

          meta = with pkgs.lib; {
            description = "Templater for daily notes";
            homepage = "https://github.com/B3ella/marmota";
            license = licenses.mit;
            maintainers = with lib.maintainers; [ b3ella ];
            mainProgram = "marmota";
          };
        };

      nixosModule = { config, lib, pkgs, ... }:
        let
          cfg = config.services.marmota;
          marmotaPkg = self.packages.${pkgs.system}.default;
        in {
          options.services.marmota = {
            enable = lib.mkEnableOption "Marmota daily note generator";

            user = lib.mkOption {
              type = lib.types.str;
              description = "User that runs Marmota.";
            };
          };

          config = lib.mkIf cfg.enable {
           environment.systemPackages = [ marmotaPkg ];
           systemd.timers.marmota = {
              wantedBy = [ "timers.target" ];

              timerConfig = {
                OnCalendar = "daily";
                Persistent = true;
              };
            };

            systemd.services.marmota = {
              description = "Generate daily notes with Marmota";

              serviceConfig = {
                Type = "oneshot";
                User = cfg.user;
                        ExecStart = "${marmotaPkg}/bin/marmota";
              };
            };
          };
        };
    in
      (flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          packages = {
            default = mkMarmotaPackage pkgs;
          };
	  checks = nixpkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            module-installs-command =
              let
                eval = nixpkgs.lib.nixosSystem {
                  modules = [
                    self.nixosModules.default
                    {
                      nixpkgs.hostPlatform = system;
                      services.marmota = {
                        enable = true;
                        user = "marmota";
                      };
                      users.users.marmota.isNormalUser = true;
                      boot.loader.grub.enable = false;
                      fileSystems."/" = {
                        device = "none";
                        fsType = "tmpfs";
                      };
                      system.stateVersion = "25.11";
                    }
                  ];
                };
                installed = builtins.elem self.packages.${system}.default
                  eval.config.environment.systemPackages;
              in
              assert nixpkgs.lib.assertMsg installed
                "services.marmota must install the marmota command";
              pkgs.runCommand "marmota-module-installs-command" { } "touch $out";
          };
        }))
      // {
        nixosModules.default = nixosModule;
      };
}
