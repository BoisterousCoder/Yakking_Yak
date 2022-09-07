import init, {newState, onJoin, onBroadcast, onSend} from "/compiled/RustyChat.js";

init().then(() => {
	let state = newState("Alice", 12345);

	const initialGroup = addFormListener("group", false, (group) => {
		console.log(onJoin(state, group))
	});
	console.log(onJoin(state, initialGroup))

	let encryptingCheckbox = document.getElementById("isEncrypting")
	addFormListener("msg", true, (msg) => {
		if(encryptingCheckbox.checked){
			console.log(onSend(state, msg));
		}else{
			console.log(onBroadcast(state, msg));
		}
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