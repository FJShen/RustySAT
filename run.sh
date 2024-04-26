EXPORT RUST_LOG=info

for FILE in input/*
do
    cargo run $FILE vsids
done
