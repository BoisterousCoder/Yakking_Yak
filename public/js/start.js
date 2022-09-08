import init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay, onAllowTrust, onTrust, getRelation, handleTrust} from "/compiled/RustyChat.js";
import { onLogin } from "./login.js";

const msgsTypes = ['i', 's', 't', 'l', '_', 'p', 'j'];

onLogin(init(), (user) => {
	let name = user.username;
	let displayedMessages = document.getElementById('messages');
	let allowTrustButton = document.getElementById('allowTrust');
	let state = newState(name, 12345);
	let socket = io();

	function sendToServer(data){
		socket.emit(getMessageType(data), data);
	}
	document.getElementById("name").innerText = name;

	const initialGroup = addFormListener("group", false, (group) => {
		sendToServer(onJoin(state, group))
	});
	sendToServer(onJoin(state, initialGroup))

	let encryptingCheckbox = document.getElementById("isEncrypting")
	addFormListener("msg", true, (msg) => {
		if(encryptingCheckbox.checked){
			sendToServer(onSend(state, msg));
		}else{
			sendToServer(onBroadcast(state, msg));
		}
	});

	for (let msgType of msgsTypes) {
		socket.on(msgType, (msg) => {
			console.log("recived message "+msg);
			state = handleIncoming(state, msg);
			let item = document.createElement('li');
			item.innerHTML = getDisplay(state, msg);
			item.addEventListener('click', (event) => {
				let name = event.target.innerHTML.split('*').map((seg) => seg.trim()).filter((seg) => seg)[0];
				if(getRelation(state, name) == "allowedTrust"){
					state = handleTrust(state, name);
					sendToServer(onTrust(state, name));
				}
			})
			displayedMessages.prepend(item);
			window.scrollTo(0, document.body.scrollHeight);
		});
	}

	allowTrustButton.addEventListener("click",() => {
		sendToServer(onAllowTrust(state));
	});
});

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