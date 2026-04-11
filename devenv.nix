{ pkgs, ... }:

{
  packages = with pkgs; [
    pkg-config
    alsa-lib
    udev
    binaryen
    wabt
    trunk
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
  };
}