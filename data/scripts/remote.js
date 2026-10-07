document.getElementById("launchkill").addEventListener("click", newterm)

document.getElementById("send").addEventListener("click", cmdsend)

document.getElementById("consolecontainer").addEventListener("keydown", function (event){
    if (event.altKey && (event.key === 'c' || event.key === 'C')){
        if (document.getElementById("cmdtextfield") == document.activeElement){
            document.getElementById("cmdtextfield").value = "CtrlC"
            cmdsend()
        }
    }
})

document.getElementById("cmdtextfield").addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
        event.preventDefault()
        cmdsend()
    }
})

document.getElementById("filesBtn").addEventListener("click", function (redirect) {
    window.location.replace(`${windowhref}/files`)
})

var windowhref = `${window.location.href.split("/")[0]}//${window.location.href.split("/")[2]}`

let terminals = []
let curr_terminal = 0
let nbterm = 0
let auth = 0

let outputhists = fetchhists()

let cmdhists = fetchcmdhists()

async function cmdsend() {
    if (terminals[curr_terminal].status == 1) {
        console.log("cmdsend")
        cmd = document.getElementById("cmdtextfield").value.trim() + "\n"
        document.getElementById("cmdtextfield").value = ""
        terminals[curr_terminal].ws.send(cmd)
    }
}

async function openwsterm(terminal) {
    let ws = new WebSocket(`${windowhref}/ws/${nbterm}`)
    terminal.ws = ws

    terminal.ws.addEventListener("open", () => {
        console.log(`WS ${terminal.id} CONNECTED`)
        termmsg(terminal.id, "CONNECTED\n")
        terminal.status = 1
        updateclass(terminal.id, 1)
        setInterval(() => {
            ws.send('ping');
            console.log(`WS ${terminal.id}: ping`);
        }, 25000);
    })
    terminal.ws.addEventListener("close", () => {
        console.log(`WS ${terminal.id} DISCONNECTED`)
        termmsg(terminal.id, "DISCONNECTED\n")
        terminal.status = 2
        updateclass(terminal.id, 2)
    });
    terminal.ws.addEventListener("error", (e) => {
        console.log(`ERROR: ${e}`)
        termmsg(terminal.id, e)
        terminal.status = 2
        updateclass(terminal.id, 2)
    });
    terminal.ws.addEventListener("message", (msg) => {
        //console.log(msg)
        termmsg(terminal.id, msg.data)
    })
    
    terminals[terminal.id] = terminal
}

function updateclass(id, status) {
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
    terminals.push(null)
    const terminal = {
        name: "null",
        ws: null,
        id: nbterm,
        status: 0, // 0: Connecting, 1: Connected, 2: Err
        output: ""
    }
    document.getElementById("terminalselector").innerHTML += `<button class="terminalbtn" id="terminalbtn${nbterm}" onclick="terminalselect(${nbterm})">${nbterm + 1}</button>`
    await openwsterm(terminal)
    terminalselect(nbterm)
    nbterm++

}

function termmsg(id, msg) {
    terminal = terminals[id]
    terminal.output += msg
    if (id == curr_terminal) {
        let outputdiv = document.getElementById("output")
        outputdiv.innerHTML += msg
        if (Math.round(outputdiv.scrollHeight-outputdiv.scrollTop/1.1) == outputdiv.offsetHeight){
            outputdiv.scrollTop = outputdiv.scrollHeight
        }
    }
}

async function upgrade() {
    fetch(`/upgrade`, {
        method: "GET"
    })
}

function terminalselect(term) {
    document.getElementById(`terminalbtn${curr_terminal}`).style.borderColor = "#100000"
    curr_terminal = term
    document.getElementById(`terminalbtn${curr_terminal}`).style.borderColor = "#ffffff"
    document.getElementById("output").innerHTML = terminals[term].output
    document.getElementById("output").scrollTop = document.getElementById("output").scrollHeight
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
        if (hists[i] == "OFF") {
            document.getElementById("terminalselector").innerHTML += `<button class="terminalbtnOFF" id="terminalbtn${term}" onclick="terminalselect(${term})">${term + 1}</button>`

        } else {
            await newterm()
            termmsg(i, hists[i])
        }
    }
    if (hists.length == 1) {
        newterm()
    } else {
        nbterms = hists.length
    }
    return hists
}

async function fetchcmdhists() {
    const response = await fetch("/cmdhists");
    if (!response.ok) {
        console.error("bad cmdhistory response")
        return
    }
    let text = await response.text()
    let hists = JSON.parse(text)
    return hists
}
