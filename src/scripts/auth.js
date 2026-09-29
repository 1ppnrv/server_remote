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
        } else if (cookie.indexOf("remote_mrbap=") === 0) {
            try {
                cookievalue = decodeURIComponent(cookie.substring(13))
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
    window.location.href = new URL(`${windowhref}`)
}

var cookievalue;
var windowhref = `${window.location.href.split("/")[0]}//${window.location.href.split("/")[2]}`

authenticate()