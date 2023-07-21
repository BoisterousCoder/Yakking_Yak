# Yakking Yak
Yakking Yak is an end to end encrypted group chat app built with Rust on Web Assembly and Node JS. 

## What is it:

It is a group chat app that uses elliptic-curve diffie hellman to set up an AES-GCM 256 secure channel between devices. 

By default, other users are untrusted, and each message is sent encrypted with a specific target user’s shared key. 

## How do you use it:

This app depends on the Rust programming language, rust web assembly compiler targets, and node js.

It uses port 4000 to communicate with the server, so it will have to be opened locally.

## Required Libraries for building
Web: [Wasm Pack](https://rustwasm.github.io/wasm-pack/)
Local: [Libadwaita](https://gtk-rs.org/gtk4-rs/stable/latest/book/libadwaita.html#libadwaita)

Note: Inorder to run the local version, the web must built and run but the reverse isn't true. This is because the local version needs the webserver to bounce messages off of.

### To build:
  
Clone this repository
1. Run `$ npm run install` to install required nodejs libraries
1. Run `$ npm run build` to build the rust web assembly library to build the pug files to html
1. Run `$ npm run start` to start the node server   

### How to Use
1. navigate to `localhost:4000` where `{username}` is a unique username of your choise
2. when the toggle switch in the bottom right is off use the text box and send button at the bottom to send unencrypted messages
3. to allow someone to trust you click the allow trust button in the top right
4. once someone has allowed you to trust them, click on their name to trust them
5. once you have trusted one or more people and they have trusted you back you can send a message using the bottom text box with the toggle switch turned on to send them an encrypted message
6. usse the text box and the go button in the top right to change your chat room
    - note: for now you will have to re-establish trust when entering new chat rooms


## Next steps
Currently the chat log is ephemeral, I am developing a distributed chat log using CRDT to make a persistent log.

Currently the messages are encrypted with a shared key, but not authenticated. I am implementing message signing based on the elliptic curve digital signature algorithm (ECDSA).

Both of these should be solved when I add in my Rusty Log project and The web native api

Currently Messages are sent to a server to be sent to out to everyone. I plan on changing this to be peer to peer. 

## Troubleshooting
You may need to create a bin folder in the public folder for the program to build into
