# Run 100 times for performance check ~ 84 seconds for 100 runs is fastest
count=100
for i in $(seq $count); do
    cargo run --release
done