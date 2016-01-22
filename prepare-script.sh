set -ex

mkdir -p .cargo
echo "[target.arm-unknown-linux-gnueabihf]" >> .cargo/config
echo "ar = \"arm-linux-gnueabihf-gcc-ar\"" >> .cargo/config
echo "linker = \"arm-linux-gnueabihf-gcc\"" >> .cargo/config

dir=rust-std-$TARGET
pkg=rust-std

curl -s $MAIN_TARGETS/$pkg-$TRAVIS_RUST_VERSION-$TARGET.tar.gz | \
tar xzf - -C $HOME/rust/lib/rustlib --strip-components=4 \
  $pkg-$TRAVIS_RUST_VERSION-$TARGET/$dir/lib/rustlib/$TARGET
