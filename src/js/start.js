//import init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay, onAllowTrust, onTrust, getRelation, handleTrust} from "../bin/RustyChat.js";
//import { onLogin } from "./login.js";
const onLogin = require("./login");
//const init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay, onAllowTrust, onTrust, getRelation, handleTrust} =
//	require('../../public/bin/puppy_talk');
const io = require("socket.io-client");
const rustCrate = require("rust_crate");

const msgsTypes = ['i', 's', 't', 'l', '_', 'p', 'j'];

onLogin((user) => {
	rustCrate.then((rust) => {
		rust.default().then(() =>{
			start(user, rust);
		});
	});
});

function start(user, rust){
	let name = user.username;
	let displayedMessages = document.getElementById('messages');
	let allowTrustButton = document.getElementById('allowTrust');
	let state = rust.newState(name, 12345);
	let socket = io();

	function sendToServer(data){
		socket.emit(getMessageType(data), data);
	}
	document.getElementById("name").innerText = name;

	const initialGroup = addFormListener("group", false, (group) => {
		sendToServer(rust.onJoin(state, group))
	});
	sendToServer(rust.onJoin(state, initialGroup))

	let encryptingCheckbox = document.getElementById("isEncrypting")
	addFormListener("msg", true, (msg) => {
		if(encryptingCheckbox.checked){
			sendToServer(rust.onSend(state, msg));
		}else{
			sendToServer(rust.onBroadcast(state, msg));
		}
	});

	for (let msgType of msgsTypes) {
		socket.on(msgType, (msg) => {
			console.log("recived message "+msg);
			state = rust.handleIncoming(state, msg);
			let item = document.createElement('li');
			item.innerHTML = rust.getDisplay(state, msg);
			item.addEventListener('click', (event) => {
				let name = event.target.innerHTML.split('*').map((seg) => seg.trim()).filter((seg) => seg)[0];
				if(rust.getRelation(state, name) == "allowedTrust"){
					state = rust.handleTrust(state, name);
					sendToServer(rust.onTrust(state, name));
				}
			})
			displayedMessages.prepend(item);
			window.scrollTo(0, document.body.scrollHeight);
		});
	}

	allowTrustButton.addEventListener("click",() => {
		sendToServer(rust.onAllowTrust(state));
	});
}

function addFormListener(name, isReseting, callback){
	let form = document.getElementById(name+'Form');
	let input = document.getElementById(name+'Input');

	form.addEventListener('submit', function(e) {
	e.preventDefault();
		if (input.value) {
			callback(input.value);
			if(isReseting){
				input.value = '';
			}
		}
	});
	return input.value;
}

function getMessageType(msg){
	return msg.split('*').map((seg) => seg.trim()).filter((seg) => seg)[1];
}