{ pkgs, lib, ... }:

{
  imports = [ ./native.nix ];

  env = {
    # Extend native LD_LIBRARY_PATH with WSL2 D3D12/dxcore libs
    LD_LIBRARY_PATH = lib.mkForce (pkgs.lib.makeLibraryPath [
      pkgs.vulkan-loader
      pkgs.libxkbcommon
      pkgs.wayland
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXi
      pkgs.xorg.libXrandr
    ] + ":/usr/lib/wsl/lib");

    # Use Mesa's dozen driver (Vulkan -> D3D12 -> host GPU via WSL dxg)
    VK_ICD_FILENAMES = "${pkgs.mesa}/share/vulkan/icd.d/dzn_icd.x86_64.json";

    # dzn is not a fully conformant Vulkan implementation, so wgpu hides it by default
    WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER = "1";
  };
}
