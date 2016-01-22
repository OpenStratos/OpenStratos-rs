set -ex

dir=rust-std-$TARGET
pkg=rust-std

if [ "$TRAVIS_RUST_VERSION" = "1.0.0" ]; then
    pkg=rust
    dir=rustc
fi
curl -s $MAIN_TARGETS/$pkg-$TRAVIS_RUST_VERSION-$TARGET.tar.gz | \
tar xzf - -C $HOME/rust/lib/rustlib --strip-components=4 \
  $pkg-$TRAVIS_RUST_VERSION-$TARGET/$dir/lib/rustlib/$TARGET
