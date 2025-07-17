# `didemo`: the Digital Identity Demonstrator

`didemo` simulates digital identity deployments by running a set of independent processes, each
simulating an actor. The actors then communicate over HTTP to simulate identity interactions, such
as issuance or age verification.

## Getting started

`docker compose` is used to run the various actors in individual containers and to manage a network
they can talk to each other on. To get started:

- Build Docker image containing all the actors: `docker buildx build . --tag didemo-actors:latest`
- Launch the actors: `docker compose -f orchestration/compose.yaml up`
- Run the tests in the `didemo_simulations` package to simulate various interactions of interest:
  `cargo test --package simulations`

## Simulation actors

TODO: describe the actors

### Persons

### Jurisdictions

### Issuers

### Relying parties

### Auditors

### Wallets

### Wallet makers

## BYO simulation actors

You can swap out any protocol actor with a different implementation, provided it implements the
expected HTTP RPC interface. That's documented in each actor's crate.
