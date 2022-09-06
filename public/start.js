import init, {newState, getJoin} from "/compiled/RustyChat.js";

init().then(() => {
    let state = newState("Alice", 12345);
    console.log(state);

    console.log(getJoin(state, "TestGroup"))
});