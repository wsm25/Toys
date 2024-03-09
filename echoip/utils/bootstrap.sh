## rustup
curl https://sh.rustup.rs | sh -s - -y --default-toolchain none
sudo apt install build-essential
## install rust
for arch in "x86_64" "i686" "aarch64"; do
    sudo apt install -y g++-${arch}-linux-gnu
    rustup target add ${arch}-unknown-linux-gnu ${arch}-unknown-linux-musl
done

    