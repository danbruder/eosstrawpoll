all: install build test

install:
	git submodule update --init --recursive
	curl https://sh.rustup.rs -sSf | sh
	rustup target add wasm32-unknown-unknown
	rustup install nightly
	rustup default nightly
	cargo install --force wasm-gc
	cargo install --force bindgen
	yarn install

build: build-contract website

test:
	cargo test --features test

clean:
	rm -Rf target

docker:
	docker-compose down
	docker volume rm -f nodeos-data-volume
	docker volume rm -f keosd-data-volume
	docker volume create --name=nodeos-data-volume
	docker volume create --name=keosd-data-volume
	docker-compose up

CLEOS := docker-compose exec keosd cleos --url http://nodeosd:8888 --wallet-url http://127.0.0.1:8900
PUBKEY := EOS6MRyAjQq8ud7hVNYcfnVPJqcVpscN5So8BhtHuGYqET5GDW5CV
PRIVKEY := 5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3

wallet:
	$(CLEOS) wallet create --to-console
	$(CLEOS) wallet import --private-key $(PRIVKEY)
	$(CLEOS) create account eosio eosstrawpoll $(PUBKEY) $(PUBKEY)
	$(CLEOS) create account eosio alice $(PUBKEY) $(PUBKEY)
	$(CLEOS) create account eosio bob $(PUBKEY) $(PUBKEY)
	$(CLEOS) create account eosio carol $(PUBKEY) $(PUBKEY)

%_gc.wasm: %.wasm
	wasm-gc $*.wasm $*_gc.wasm

%_gc_opt.wasm: %_gc.wasm
	wasm-opt --fuzz-exec --output $*_gc_opt.wasm -Oz $*_gc.wasm

%_gc_opt.wat: %_gc_opt.wasm
	wasm2wat $*_gc_opt.wasm -o $*_gc_opt.wat --generate-names

%_gc_opt_wat.wasm: %_gc_opt.wat
	wat2wasm $*_gc_opt.wat -o $*_gc_opt_wat.wasm

# --------
# CONTRACT
# --------

build-contract: build-contract-wasm target/wasm32-unknown-unknown/release/contract_gc_opt_wat.wasm

build-contract-wasm:
	cargo build --release --target=wasm32-unknown-unknown --package contract

contract: build-contract
	$(CLEOS) set abi eosstrawpoll /mnt/dev/contract/contract.abi.json
	$(CLEOS) set code eosstrawpoll /mnt/dev/release/contract_gc_opt.wasm

createpoll:
	$(CLEOS) push action eosstrawpoll createpoll '["test","alice","This is a test poll",[],1,1,1,true,[],0,0]' -p 'alice@active'

destroypoll:
	$(CLEOS) push action eosstrawpoll destroypoll '["test"]' -p 'alice@active'

getpolls:
	$(CLEOS) get table eosstrawpoll eosstrawpoll polls

getvotes:
	$(CLEOS) get table eosstrawpoll eosstrawpoll votes

getpopular:
	$(CLEOS) get table eosstrawpoll eosstrawpoll popularpolls

getnew:
	$(CLEOS) get table eosstrawpoll eosstrawpoll newpolls

# -------
# WEBSITE
# -------

NPM := ./node_modules/.bin
PWD := `pwd`
DIST := $(PWD)/dist

website: website-wasm css webpack

website-wasm: website.wasm target/wasm32-unknown-unknown/release/website_gc_opt_wat.wasm

website.wasm:
	cargo web build --release --target=wasm32-unknown-unknown --package website

start-website-wasm:
	cargo watch -w website/src -s "make website-wasm"

css:
	$(NPM)/postcss website/static/css/index.css --output $(DIST)/index.css

start-css:
	$(NPM)/postcss website/static/css/index.css --output $(DIST)/index.css --watch

webpack:
	$(NPM)/env-cmd .env.prod $(NPM)/webpack

start-webpack:
	$(NPM)/env-cmd .env.dev $(NPM)/webpack --watch

start-server:
	$(NPM)/browser-sync start --single --config bs-config.js

start-website:
	$(NPM)/concurrently \
		--raw \
		--kill-others \
		"$(MAKE) start-website-wasm" \
		"$(MAKE) start-webpack" \
		"$(MAKE) start-css" \
		"$(MAKE) start-server"

gh-pages: build
	git worktree remove ./gh-pages || exit 0
	git worktree add ./gh-pages gh-pages
	mv dist/* gh-pages
	cp netlify.toml ./gh-pages/

.PHONY: install build test clean docker wallet website gh-pages
.SECONDARY:
