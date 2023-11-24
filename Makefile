.PHONY: gen-deploy generate

gen-deploy:
	./did.sh && dfx generate && dfx deploy -y --playground

generate:
	./did.sh && dfx generate
