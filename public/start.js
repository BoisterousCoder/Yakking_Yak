import init, {newState, onJoin, onBroadcast, onSend, handleIncoming, getDisplay} from "/compiled/RustyChat.js";

const msgsTypes = ['i', 's', 't', 'l', '_', 'p', 'j'];

init().then(() => {
	let name = document.getElementById("name").innerHTML;
	var displayedMessages = document.getElementById('messages');
	let state = newState(name, 12345);
	let socket = io();

	function sendToServer(data){
		socket.emit(getMessageType(data), data);
	}

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
			item.textContent = getDisplay(msg);
			displayedMessages.appendChild(item);
			window.scrollTo(0, document.body.scrollHeight);
		});
	}
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