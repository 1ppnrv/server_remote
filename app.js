// Filename - index.js

// Importing express module
const express = require("express")
const fs = require("fs")
const app = express()

// Handling GET / request
app.get("/", (req, res, next) => {
	console.log("Request for root")
	res.send("This is the express server")
})

// Handling GET /hello request
app.get("/remote", (req, res, next) => {
	console.log("Request for remote")
	res.set("Content-Type", "text/html");
	fs.readFile("src/console.html", (err, data) => {
		if (err) {
			console.log(`Error : ${err}`)
			return;
		}
		res.status(200)
		res.send(data)
	})
	
})

// Server setup
app.listen(10001, () => {
	console.log("Server is Running")
})