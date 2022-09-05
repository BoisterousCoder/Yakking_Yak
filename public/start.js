import init, {onConnect} from "/compiled/RustyChat.js";

init().then(() => {
    let state = onConnect("Alice", 12345, "test");
    console.log(state);
});