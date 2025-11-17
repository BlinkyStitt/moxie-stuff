# Moxie Stuff

Scripts to manage your Moxie.

Both Moxie and Frames v1 are gone. So this project isn't very useful.

## Setup

1. Get a [Neynar API Key](https://dev.neynar.com/).

2. Read the docs

   cargo doc --open

3. Copy the example environment file and then fill in your information

   cp env.example .env

4. Claim your everyday rewards and buy more fan tokens with them:

   cargo run --release

## Development

Download the latest graphql schemas:

```shell
graphql-client introspect-schema https://bff-prod.airstack.xyz/graphql > graphql/airstack-bff-prod/schema.json
```

```shell
graphql-client introspect-schema https://airstack.xyz/api/protocol-subgraph > graphql/airstack-protocol-subgraph/schema.json
```

NOTE: The schemas for airstack-claims were built by hand and are probably wrong. They didn't grant me API access because this is a personal project.
