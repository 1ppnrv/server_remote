document.getElementById("launch").addEventListener("click", newterm)

var windowhref = `${window.location.href.split("/")[0]}//${window.location.href.split("/")[2]}`

let terminals = []
let curr_terminal = 0
let nbterm = 0
let auth = 0

let outputhists = fetchhists()

async function openwsterm(terminal) {
    let ws = new WebSocket(`${windowhref}/ws/${nbterm}`)
    terminal.ws = ws

    terminal.ws.addEventListener("open", () => {
        console.log(`WS ${terminal.id} CONNECTED`)
        terminal.status = 1
        updateclass(terminal, 1)
        setInterval(() => {
            ws.send('ping');
            console.log(`WS ${terminal.id}: ping`);
        }, 25000);
    })
    terminal.ws.addEventListener("close", () => {
        console.log(`WS ${terminal.id} DISCONNECTED`)
        terminal.terminal.write("DISCONNECTED")
        terminal.status = 2
        updateclass(terminal, 2)
    });
    terminal.ws.addEventListener("error", (e) => {
        console.log(`ERROR: ${e}`)
        terminal.terminal.write(msg)
        terminal.status = 2
        updateclass(terminal, 2)
    });
    terminal.ws.addEventListener("message", (msg) => {
       terminal.terminal.write(msg.data)
    })
    
    terminals[terminal.id] = terminal
}

function updateclass(terminal, status) {
    if (status == 0) {
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnOFF")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtnOFF")
        }
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnON")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtnON")
        }
        if (!document.getElementById(`terminalbtn${terminal.id}`).classList.contains("treminalbtn")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.add("terminalbtn")
        }
    } else if (status == 1) {
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnOFF")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtnOFF")
        }
        if (!document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnON")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.add("terminalbtnON")
        }
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("treminalbtn")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtn")
        }
    } else if (status == 2) {
        if (!document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnOFF")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.add("terminalbtnOFF")
        }
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("terminalbtnON")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtnON")
        }
        if (document.getElementById(`terminalbtn${terminal.id}`).classList.contains("treminalbtn")) {
            document.getElementById(`terminalbtn${terminal.id}`).classList.remove("terminalbtn")
        }
    }
}

async function newterm() {
    const terminal = {
        name: "null",
        ws: null,
		terminal: new Terminal(),
        id: nbterm,
        status: 0, // 0: Connecting, 1: Connected, 2: Err
        output: ""
    }

	const new_button = document.createElement("button")
	new_button.id = `terminalbtn${nbterm}`
	new_button.className = "terminalbtn"
	new_button.textContent = `${nbterm + 1}`
	new_button.onclick = function () {
		terminalselect(terminal.id)
	}
	document.getElementById("terminalselector").appendChild(new_button)

	const newterminaldiv = document.createElement("div")
	newterminaldiv.id = `terminal_${nbterm}`
	newterminaldiv.className = "terminal"
	document.getElementById("consolecontainer").appendChild(newterminaldiv)


	terminal.terminal.open(document.getElementById(`terminal_${nbterm}`))
	terminal.terminal.onData(data => {
		terminal.ws.send(data);
	});
	openwsterm(terminal)
	document.getElementById(`terminal_${nbterm}`).style.display = "none"
	terminalselect(nbterm)
	terminals.push(terminal)
	console.log(`create terminal ${nbterm}`)
    nbterm++

}

async function upgrade() {
    fetch(`/upgrade`, {
        method: "GET"
    })
}

function terminalselect(term) {
    document.getElementById(`terminalbtn${curr_terminal}`).style.borderColor = "#100000"
	document.getElementById(`terminal_${curr_terminal}`).style.display = "none"
    curr_terminal = term
	document.getElementById(`terminal_${curr_terminal}`).style.display = "block"
    document.getElementById(`terminalbtn${curr_terminal}`).style.borderColor = "#ffffff"
}

async function fetchhists() {
    const response = await fetch("/outputhist");
    if (!response.ok) {
        console.error("bad history response")
        nbterm = 0
        return
    }
    let text = await response.text()
    let hists = text.split("endterm")

    for (let i = 0; i < hists.length - 1; i++) {
		await newterm()
		text = hists[hists.length-2-i]
		terminals[i].terminal.write(text)
    }
    if (hists.length == 1) {
        await newterm()
    }
    nbterms = hists.length
    return hists
}