# this script builds release artifact zip files for linux, linux-nixos and windows
# requires programs defined in dev shell 'buildArtifact', enable with 'nix develop .#buildArtifact'

# set version for artifact zip names
VERSION=v1.8.0

# create directory
rm -r artifacts
mkdir artifacts

# build linux nixos zip
echo "building nixos artifact"
cargo build --release
cp target/release/alpha_tui .
zip -r artifacts/alpha_tui-$VERSION-linux-nixos.zip alpha_tui
zip -r artifacts/alpha_tui-$VERSION-linux-nixos.zip LICENSE
zip -r artifacts/alpha_tui-$VERSION-linux-nixos.zip examples/
zip -r artifacts/alpha_tui-$VERSION-linux-nixos.zip themes/
# cleanup binary
rm alpha_tui

# build linux zip
echo "building linux artifact"
# if this failes first install default toolchain with 'rustup toolchain add stable'
cross build --target x86_64-unknown-linux-gnu --release
cp target/x86_64-unknown-linux-gnu/release/alpha_tui .
zip -r artifacts/alpha_tui-$VERSION-linux.zip alpha_tui
zip -r artifacts/alpha_tui-$VERSION-linux.zip LICENSE
zip -r artifacts/alpha_tui-$VERSION-linux.zip examples/
zip -r artifacts/alpha_tui-$VERSION-linux.zip themes/
# cleanup binary
rm alpha_tui

# build windows zip
echo "building windows artifact"
nix build .#alpha_tui-win
cp result/bin/alpha_tui.exe .
zip -r artifacts/alpha_tui-$VERSION-windows.zip alpha_tui.exe
zip -r artifacts/alpha_tui-$VERSION-windows.zip LICENSE
zip -r artifacts/alpha_tui-$VERSION-windows.zip examples/
zip -r artifacts/alpha_tui-$VERSION-windows.zip themes/
rm -f alpha_tui.exe

echo "release artifacts have been build and placed in artifacts/"
