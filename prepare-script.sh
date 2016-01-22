set -ex

mkdir -p .cargo
echo "[target.arm-unknown-linux-gnueabihf]" >> .cargo/config
echo "ar = \"arm-linux-gnueabihf-gcc-ar\"" >> .cargo/config
echo "linker = \"arm-linux-gnueabihf-gcc\"" >> .cargo/config

dir=rust-std-arm-unknown-linux-gnueabihf
tarball=https://static.rust-lang.org/dist/rust-std-$TRAVIS_RUST_VERSION-arm-unknown-linux-gnueabihf.tar.gz

curl -s $tarball | \
tar xzf - -C $HOME/rust/lib/rustlib --strip-components=4 \
  rust-std-$TRAVIS_RUST_VERSION-arm-unknown-linux-gnueabihf/$dir/lib/rustlib/arm-unknown-linux-gnueabihf
