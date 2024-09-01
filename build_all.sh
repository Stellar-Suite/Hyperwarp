cd hyperwarp
make all || exit 1
echo Built hyperwarp
cd ..
cd streamerd
cargo build || exit 1
cargo build --release || exit 1
echo Built streamerd