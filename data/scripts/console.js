document.getElementById("cmdtextfield").addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
        event.preventDefault();
        cmdsend();
    }
});

async function authenticate() {
    const cookiearray = document.cookie.split(';')
    for (let i = 0; i < cookiearray.length; i++) {
        let cookie = cookiearray[i]
        cookie = cookie.trim()

        if (cookie.indexOf("auth=") === 0) {
            try {
                cookievalue = decodeURIComponent(cookie.substring(5))
                const response = await fetch("/login/cookie", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        password: cookievalue
                    })
                });
                if (!response.ok) {
                    console.error(`HTTP error! Status: ${response.status}`);
                }
                const result = await response.json();

                if (result.login_state != 1) {
                    window.location.replace("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                }
                auth = 1
                return

            } catch (error) {
                console.error("Error sending POST request:", error);
            }
        } else if (cookie.indexOf("remote_console=") === 0) {
            try {
                cookievalue = decodeURIComponent(cookie.substring(15))
                const response = await fetch("/login/cookie", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        password: cookievalue
                    })
                });
                if (!response.ok) {
                    console.error(`HTTP error! Status: ${response.status}`);
                }
                const result = await response.json();

                if (result.login_state != 1) {
                    window.location.href = "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                }
                auth = 1
                return

            } catch (error) {
                console.error("Error sending POST request:", error);
            }
        }
    }
    window.location.href = new URL(`https://${window.location.href.split("/")[2]}`)
}

async function cmdsend() {
    if (terminals[curr_terminal].status == 1) {
        console.log("cmdsend")
        cmd = document.getElementById("cmdtextfield").value.trim() + "\n"
        document.getElementById("cmdtextfield").value = ""
        terminals[curr_terminal].ws.send(cmd)
    }
}

async function openwsterm(terminal) {
    let ws = new WebSocket(`http://${window.location.href.split("/")[2]}/ws/${nbterm}`)
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
        if (Math.round(outputdiv.scrollHeight-outputdiv.scrollTop) == outputdiv.offsetHeight){
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

async function fetchterm() {
    const response = await fetch("/hist");
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

let terminals = []
let curr_terminal = 0
let nbterm = 0
let auth = 0
let cookievalue

authenticate()
let hists = fetchterm()
