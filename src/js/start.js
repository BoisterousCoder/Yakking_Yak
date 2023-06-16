//import init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay, onAllowTrust, onTrust, getRelation, handleTrust} from "../bin/RustyChat.js";
//import { onLogin } from "./login.js";
const onLogin = require("./login");
//const init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay, onAllowTrust, onTrust, getRelation, handleTrust} =
//	require('../../public/bin/puppy_talk');
const io = require("socket.io-client");
const rustCrate = require("rust_crate");

const msgsTypes = ['i', 's', 't', 'l', '_', 'p', 'j'];

onLogin((user) => {
	rustCrate.then((_rust) => {
		window.rust = _rust;
		window.rust.default().then(() =>{
			start(user, window.rust);
		});
	});
});

function start(user, rust){
	let name = user.username;
	
	let socket = io();
	window.sendToServer = (data) =>{
		console.log("Sending Msg "+ data);
		socket.emit(getMessageType(data), data);
	}
	window.rustState = rust.createState(name);

	for (let msgType of msgsTypes) {
		socket.on(msgType, (msg) => {
			console.log("recived message "+msg);
			window.rustState = window.rust.handleEvent(window.rustState, window.sendToServer, "msg-"+msgType, msg);
			//eventResults.push(["msg-"+msgType, msg])
			// state = rust.handleIncoming(state, msg);
			// let item = document.createElement('li');
			// item.innerHTML = rust.getDisplay(state, msg);
			// item.addEventListener('click', (event) => {
			// 	let name = event.target.innerHTML.split('*').map((seg) => seg.trim()).filter((seg) => seg)[0];
			// 	if(rust.getRelation(state, name) == "allowedTrust"){
			// 		state = rust.handleTrust(state, name);
			// 		sendToServer(rust.onTrust(state, name));
			// 	}
			// })
			// displayedMessages.prepend(item);
			// window.scrollTo(0, document.body.scrollHeight);
		});
	}
	document.getElementById("isEncrypting").checked = false;

	// dochandlingument.getElementById("name").innerText = name;

	// const initialGroup = addFormListener("group", false, (group) => {
	// 	sendToServer(rust.onJoin(state, group))
	// });
// 	sendToServer(rust.onJoin(state, initialGroup))

// 	let encryptingCheckbox = document.getElementById("isEncrypting")
// 	addFormListener("msg", true, (msg) => {
// 		if(encryptingCheckbox.checked){
// 			sendToServer(rust.onSend(state, msg));
// 		}else{
// 			sendToServer(rust.onBroadcast(state, msg));
// 		}
// 	});

// 	

// 	allowTrustButton.addEventListener("click",() => {
// 		sendToServer(rust.onAllowTrust(state));
// 	});
// }

}

function getMessageType(msg){
	return msg.split('*').map((seg) => seg.trim()).filter((seg) => seg)[1];
}