{
    pkgs ? import <nixpkgs> { },
}:
let
    overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
    libPath =
        with pkgs;
        lib.makeLibraryPath [
            # load external libraries that you need in your rust project here
        ];

    treesitter-overlay = final: prev: {
        tree-sitter = prev.tree-sitter.overrideAttrs (prevAttrs: rec {
            version = "0.26.6";
            src = pkgs.fetchurl {
                url = "https://github.com/tree-sitter/tree-sitter/archive/refs/tags/v${version}.tar.gz";
                sha256 = "sha256-tCGBhaSKeR1AIqs5aXCeJxpwoCU+lHkqu88Y1/z0KRw=";
            };
            patches = [];
            env = prevAttrs.env // {
                LIBCLANG_PATH = "${lib.getLib final.llvmPackages.libclang}/lib";
            };
            nativeBuildInputs = prevAttrs.nativeBuildInputs ++ [
                final.rustPlatform.bindgenHook
            ];
            cargoDeps = prev.rustPlatform.fetchCargoVendor {
                inherit src;
                name = "${prevAttrs.pname}-${version}-vendor";
                hash = "sha256-u6RmwNR4QVwyuij5RlHTLC5lNNQpWMVrlQwfwF78pYc=";
            };
        });
    };
    sources = import ./npins;
    newPkgs = import <nixpkgs> {
        overlays = [ treesitter-overlay ];
    };
    lib = pkgs.lib;
in
pkgs.mkShell rec {
    buildInputs = with pkgs; [
        clang
        llvmPackages_21.bintools
        rustup
        newPkgs.tree-sitter
        pkg-config
        openssl
        tree-sitter
        eslint
        nodejs
    ];
    RUSTC_VERSION = overrides.toolchain.channel;
    LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
    shellHook = ''
        export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
        export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    '';
    RUSTFLAGS = (
        builtins.map (a: ''-L ${a}/lib'') [
        ]
    );
    LD_LIBRARY_PATH = libPath;
    BINDGEN_EXTRA_CLANG_ARGS =
        (builtins.map (a: ''-I"${a}/include"'') [ pkgs.glibc.dev])
        ++ 
        [
            ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
            ''-I"${pkgs.glib.dev}/include/glib-2.0"''
            ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
        ];
}

