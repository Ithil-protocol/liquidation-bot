CONTAINER_REGISTRY_URL=eu.gcr.io
PROJECT_ID=ithil-bots-359309
SERVICE=goerli-liquidation-bot

.PHONY: build
build:
	cargo build

.PHONY: format
format:
	cargo fmt

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test

.PHONY: build-docker-image
build-docker-image:
	docker build . \
	--iidfile .dockeriid \
	--tag $(CONTAINER_REGISTRY_URL)/$(PROJECT_ID)/$(SERVICE):latest

.PHONY: push-image-to-container-registry
push-image-to-container-registry:
	docker push $(CONTAINER_REGISTRY_URL)/$(PROJECT_ID)/$(SERVICE):latest
