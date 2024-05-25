{
  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system: let
    pkgs = import nixpkgs { inherit system; };
  in {
    devShells.default = pkgs.mkShell {
      buildInputs = with pkgs; [ nixVersions.latest ];
      nativeBuildInputs = with pkgs; [ rustPlatform.bindgenHook ];
    };
  });
}
