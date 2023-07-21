const express = require('express');
const http = require('http');
const socket = require("socket.io");

const msgsTypesToRepeat = ['i', 's', 't', 'l', '_', 'p'];
const hostname = process.env.HOSTNAME||'localhost';
const port = process.env.PORT||4000;

const app = express();
const server = http.createServer(app);
const io = new socket.Server(server);
app.use('/', express.static('public'))

io.on('connection', (socket) => {
	let room;
	let user;
	console.log('a user connected');
	for (let msgType of msgsTypesToRepeat) {
		socket.on(msgType, (msg) => {
			if (room && cleanMsg(msg)[1] == msgType){
				msg += Date.now() + "*"
				console.log("user sent msg " + msg + " to room "+room);
				io.to(room).emit(msgType, msg);
			}
		});
	}
	socket.on('j', (msg) => {
		let leaveMsg;
		let oldRoom = room;
		if (room && user) {
			leaveMsg = getLeaveMsg();
		}
		let parts = cleanMsg(msg);
		if(parts[1] == 'j'){
			if(room) socket.leave(room);
			room = parts[2];
			user = parts[0];
			console.log("user joined room " +room);
			socket.join(room);
			io.to(room).emit('j', msg + Date.now() + "*");
		}
		if(leaveMsg){
			io.to(oldRoom).emit('l', leaveMsg)
		}
	});
	socket.on("disconnect", () => {
		if (room && user) {
			io.to(room).emit('l', getLeaveMsg())
		}
	});

	function getLeaveMsg(){
		console.log("Getting leave msg");
		return "*"+user+"*l*"+room+"*"+Date.now()+"*"
	}
});

server.listen(port, hostname, () => {
	console.log(`Server running at http://${hostname}:${port}/`)
});

function cleanMsg(msg){
	return msg.split('*').map((seg) => seg.trim()).filter((seg) => seg);
}