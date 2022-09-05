const express = require('express');
const pug = require('pug');
const http = require('http');
const socket = require("socket.io");

const hostname = '127.0.0.1';
const port = 4000;

const app = express();
const server = http.createServer(app);
const io = new socket.Server(server);
const homepage = pug.compileFile('./src/pug/index.pug');

app.get('/', (req, res) => {
	res.send(homepage({
		name: 'Savannah'
  	}));
});
app.use('/compiled', express.static('pkg'))
app.use('/static', express.static('public'))

io.on('connection', (socket) => {
	console.log('a user connected');
});

server.listen(port, hostname, () => {
	console.log(`Server running at http://${hostname}:${port}/`)
});
