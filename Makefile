build:
	cargo build

serve:
	./maelstrom/maelstrom serve

echo: build
	./maelstrom/maelstrom test -w echo --bin ./target/debug/fly --node-count 1 --time-limit 10

unique: build
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/fly --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast: build
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/fly --node-count 5 --time-limit 20 --rate 100

broadcast-fault: build
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/fly --node-count 5 --time-limit 20 --rate 10 --nemesis partition

kafka: build
	./maelstrom/maelstrom test -w kafka --bin ./target/debug/kafka --node-count 1 --concurrency 2n --time-limit 20 --rate 1000

all: echo unique broadcast
