set -ex

dir=rust-std-$TARGET
pkg=rust-std

curl -s $MAIN_TARGETS/$pkg-$TRAVIS_RUST_VERSION-$TARGET.tar.gz | \
tar xzf - -C $HOME/rust/lib/rustlib --strip-components=4 \
  $pkg-$TRAVIS_RUST_VERSION-$TARGET/$dir/lib/rustlib/$TARGET
