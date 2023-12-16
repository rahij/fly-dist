build:
	cargo build

serve:
	./maelstrom/maelstrom serve

echo: build
	./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10

unique: build
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/echo --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast: build
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/echo --node-count 5 --time-limit 20 --rate 100

broadcast-fault: build
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/echo --node-count 5 --time-limit 20 --rate 10 --nemesis partition

all: echo unique broadcast