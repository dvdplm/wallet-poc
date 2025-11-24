# "Wallet-as-a-service"

## What

Proof-of-concept implementation of a wallet-as-a-service, providing users with a tool to sign messages.

Build the project with `cargo build --release`; your local `target/release` folder will contain two binaries: `signingserver` and `sign`.

## How

The code is organized into three crates, `signingserver` and `signingclient` and some common type definitions in `signingcommon`. The server is a very simple web application (using `axum`) and the client is a CLI tool called `sign` that takes a seed and a message to be signed.

The client, a command line tool called `sign`, works like so:

1. Register a user:

```
$ sign register "my-secret-seed-here"
```

The output is a two lines of text with the UUID and the verifying key, to be stored and saved carefully by the user.

2. Sign a message:

```
$ sign -u 4c0d6763-cc53-4270-8b65-de150f55e739 -m "my message to be signed here"
```

The output is a hex encoded Ed25519 signature. See `sign --help` and `sign register --help` for further options.

When compiling the signing service from source, replace `sign` in the above with `cargo run --bin sign -- `.

### Notes

The signing server only accepts TLS connections and communicates with outside clients over a JSON api.

No data is stored on disk, the server operates entirely in memory and tries to avoid runtime memory allocation. By default the service can hold 1024 users. When the server stops, no trace is left on the host side (no log files, no user database, no signatures). Users can re-register their seeds, which will derive the same signing key (do note that the UUIDs are random, and so forgotten each time the service restarts).

Messages are signed using Ed25519. User IDs are UUID v4, providing a standard string representation and an efficient fixed size key type.

Users "register" with the service using a "seed", supplied on the command line. The seed is combined with a "master secret" and fed to a KDF (`hkdf` crate, using SHA2) to create the actual signing key. Anyone in possession of the seed can sign messages. Anyone can ask the service to "forget" a user. 


## Discussion

It's important to note that using a signing service like is a terrible idea from a security point of view. The old degen saying of "not your keys not your crypto" applies. Making something like this secure is quite difficult and constitutes a single point of failure for all users, thus defeating the idea of using decentralized tech like a blockchain. Signing messages is not a computationally difficult operation and should be implemented locally whenever possible, preferably using hardware wallets.

Short list of deficiencies in my implementation:

- No DoS protection or rate limiting.
- Master secret is hard coded in the implementation. If it is stolen, the thief can derive signing keys for any seed they possess or can guess.
- This PoC implementation uses self-signed certificates, obviously a big no-no for anything serious.
- No effort has been made to ensure signing is constant time/space.
- Large messages will likely not work. As-is and without further work it's not obvious what the limit is (network payload limits, OS-dependent limits, `axum` limits are all in play).
- The `forget/` endpoint is not protected and anyone can forget any user they know the UUID for.
- No key recovery, revocation or backup facilities. If you loose the seed, you loose access to the signing key.
- No effort was made to make the signing service performant, e.g. by sharding users and/or using a lock-free storage data structure, or batching up messages to be signed to leverage `ed25519-dalek`'s batch signing facilities (TODO CHECK).

### Design philosophy

The code here is purposefully boring. No non-standard technology choices, no nightly features, no exotic signature scheme, transport mechanism or anything else that would make the code difficult to understand. There is no configuration and no algorithm flexibility. Ed25519 and HKDF-SHA2 are well understood, battle tested and "boring" choices.

The storage uses the `heapless` crate to store keys, thus eschewing the can of worms that is guaranteeing proper zeroization of secrets in the presence of an allocator that may or may not re-use memory. This comes at the price of a hard-coded max limit on the number of users of the service.

Any half-decent developer, even without any specific cryptography or security training, should be able to read and understand the code in this repo with minimal effort.

### Defense in depth

Even if I am of the opinion that digital signatures should really be handled locally, here's a list of things that could be done to provide some defense in depth to an implementation like the one in this repo:

- Run the code in a TEE (TDX is probably the best choice).
- Add ACLs to restrict who can use the service.
- Add network restrictions to control how the service is accessed.
- Audit the code base for constant-time-ness on all OSs and hardware platforms where it is deployed.
- Add code attestation and make sure the client refuses to speak to non-authentic servers.
- Implement normal sysadmin best-practices for the server (e.g. up to date OS, minimal privileges, locked down networking stack).

Even with all of the above, I'd be hesitant to actually use a wallet-as-a-service for anything important. It's just not a great fit. 

To *actually* let a remote server sign things on the user's behalf there'd need to be a proper MCP solution, e.g. threshold signatures.
