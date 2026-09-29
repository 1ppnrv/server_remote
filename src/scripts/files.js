async function fetchfiles(path){
    let res = await fetch(`/files?=${path}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({path: path})
        }
    )
}

document.getElementById("consoleBtn").addEventListener("click", function (redirect) {
    window.location.replace(`${windowhref}/console`)
})