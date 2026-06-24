{
  description = "KR580 desktop emulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
          runtimeLibs = [
            pkgs.fontconfig
            pkgs.freetype
            pkgs.libGL
            pkgs.libxkbcommon
            pkgs.vulkan-loader
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.xorg.libxcb
          ];
          kr580 = pkgs.rustPlatform.buildRustPackage {
            pname = "kr580";
            version = cargoToml.workspace.package.version;
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                let
                  name = baseNameOf path;
                in
                !(builtins.elem name [
                  ".git"
                  ".tmp"
                  "dist"
                  "target"
                ]);
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [
              pkgs.makeWrapper
              pkgs.pkg-config
            ];
            buildInputs = runtimeLibs;
            cargoBuildFlags = [
              "-p"
              "kr580"
              "--bin"
              "k580"
              "--bin"
              "kr"
            ];
            doCheck = false;
            installPhase = ''
              runHook preInstall
              install_target_dir="target/${pkgs.stdenv.targetPlatform.rust.cargoShortTarget}/$cargoBuildType"

              install -Dm755 "$install_target_dir/k580" "$out/bin/k580"
              install -Dm755 "$install_target_dir/kr" "$out/bin/kr"
              install -Dm644 assets/icons/icon-256.png "$out/share/icons/hicolor/256x256/apps/kr580.png"
              install -Dm644 assets/icons/file-580.png "$out/share/icons/hicolor/256x256/mimetypes/application-x-kr580.png"

              install -Dm644 /dev/stdin "$out/share/applications/kr580.desktop" <<'DESKTOP'
              [Desktop Entry]
              Type=Application
              Name=KR580
              Comment=KR580VM80 / Intel 8080 emulator
              Exec=kr %f
              Icon=kr580
              Terminal=false
              Categories=Development;Emulator;
              MimeType=application/x-kr580;
              DESKTOP

              install -Dm644 /dev/stdin "$out/share/mime/packages/application-x-kr580.xml" <<'MIME'
              <?xml version="1.0" encoding="UTF-8"?>
              <mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
                <mime-type type="application/x-kr580">
                  <comment>KR580 snapshot</comment>
                  <glob pattern="*.580"/>
                </mime-type>
              </mime-info>
              MIME

              runHook postInstall
            '';
            postFixup = ''
              wrapProgram "$out/bin/k580" --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs}
              wrapProgram "$out/bin/kr" --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs}
            '';
            meta = {
              description = "Desktop KR580VM80 / Intel 8080 emulator";
              homepage = "https://github.com/WhoSowSee/KR580";
              license = lib.licenses.mit;
              mainProgram = "k580";
              platforms = systems;
            };
          };
        in
        {
          default = kr580;
          kr580 = kr580;
        });

      apps = forAllSystems (system:
        let
          package = self.packages.${system}.default;
        in
        {
          default = {
            type = "app";
            program = "${package}/bin/k580";
          };
          kr = {
            type = "app";
            program = "${package}/bin/kr";
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        });
    };
}
