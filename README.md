# RustyChat
Rusty Chat is an end to end encrypted group chat app built with Rust. 

## What is it:

It is a group chat app that uses elliptic-curve diffie hellman to set up an AES-GCM 256 secure channel between devices. 

By default, other users are untrusted, and each message is sent encrypted with a specific target user’s shared key. 

## How do you use it:

This app depends on the Rust programming language, which can be installed using rustup.

It uses port 4000 to communicate with the server, so it will have to be opened locally.

### To build:
  
Clone this repository
1. Run `$ cargo build`,
1. Run `$ cargo run --bin server` to set up the server program, and then  run `$ cargo run --bin client` to run the client program.   

### Client Commands:
- `/allowTrust` enables the trusting of users in the chat
- `/trust <Name>` trusts the user with that name, and sets up a shared key with that user.
- `/broadcast <message>` sends the message unencrypted to the group.
- `/list` lists currently trusted users

## Next steps
Currently the chat log is ephemeral, I am developing a distributed chat log using CRDT to make a persistent log.

Currently the messages are encrypted with a shared key, but not authenticated. I am implementing message signing based on the elliptic curve digital signature algorithm (ECDSA).
