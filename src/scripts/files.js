document.getElementById("uploadbtn").addEventListener("click", upload())
document.getElementById("consoleBtn").addEventListener("click", function (redirect) {
    window.location.replace(`${windowhref}/console`)
})

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

