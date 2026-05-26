document.getElementById("password").addEventListener("keydown", function (event) {
    if (event.key === "Enter") {
        event.preventDefault();
        login();
    }
});

async function autoconnect() {
    const cookiearray = document.cookie.split(';')
    for (let i = 0; i < cookiearray.length; i++) {
        let cookie = cookiearray[i]
        cookie = cookie.trim()

        if (cookie.indexOf("remote_console=") === 0) {
            try {
                const response = await fetch("/login/cookie", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({
                        password: decodeURIComponent(cookie.substring(15))
                    })
                });
                if (!response.ok) {
                    console.log(`HTTP error! Status: ${response.status}`);
                }
                const result = await response.json();

                if (result.login_state == 1) {
                    console.log("cookie connection")
                    window.location.replace(`${window.location.href}console`)
                }

            } catch (error) {
                console.error("Error sending POST request:", error);
            }

        }
    }
}

async function login() {
    password = document.getElementById("password").value.trim();
    error = document.getElementById("error");
    if (password === "") {
        error.style.display = "block";
        error.textContent = "Please enter a password";
    } else {
        error.style.display = "block";
        try {
            const response = await fetch("/login/pwd", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    password: password
                })
            });
            if (!response.ok) {
                console.log(`HTTP error! Status: ${response.status}`);
            }
            const result = await response.json();

            if (result.login_state == 1) {
                let date = new Date;
                if (document.getElementById("autoconnect").checked) {
                    date.setFullYear(date.getFullYear() + 100);
                    document.cookie = `${encodeURIComponent("remote_console")}=${encodeURIComponent(result.cookie_value)}; expires=${date.toUTCString()}; path=/; Secure; SameSite=Strict`;
                    date = new Date;
                }else{
                date.setTime(date.getTime() + 86400);
                document.cookie = `${encodeURIComponent("auth")}=${encodeURIComponent(result.cookie_value)}; expires=${date.toUTCString()}; path=/; Secure; SameSite=Strict`;
                }
                window.location.replace(`${window.location.href}console`)

            } else {
                document.getElementById("password").value = ""
                error.style.display = "block";
                error.textContent = "Wrong password";
            }

        } catch (error) {
            console.error("Error sending POST request:", error);
        }
    }
}

autoconnect()