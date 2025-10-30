.PHONY: all
all: deploy

.PHONY: deploy
.SILENT: deploy
deploy:
	dfx deploy basic_dogecoin --argument '(variant { regtest })'

.PHONY: regtest_topup
.SILENT: regtest_topup
regtest_topup:
	P2PKH_ADDR=$(shell dfx canister call basic_dogecoin get_p2pkh_address | tr -d '()') && \
	TOPUP_CMD_P2PKH_ADDR="dogecoin-cli -regtest -rpcport=8333 sendtoaddress $${P2PKH_ADDR} 1" && \
	eval "$${TOPUP_CMD_P2PKH_ADDR}" && \
	eval "$${TOPUP_CMD_P2PKH_ADDR}" && \
	eval "$${TOPUP_CMD_P2PKH_ADDR}" && \
	dogecoin-cli -regtest -rpcport=8333 -generate 6

.PHONY: clean
.SILENT: clean
clean:
	rm -rf .dfx
	rm -rf dist
	rm -rf node_modules
	rm -rf src/declarations
	rm -f .env
	cargo clean
