import { login } from "./api_client.js";
export let navigate = (p) => {
    history.pushState({}, document.title, p);
    window.dispatchEvent(new Event("popstate"));
}

export let submit = {
    "login": async (event) => {
        event.preventDefault();
        let uname = document.getElementById("username").value;
        let passwd = document.getElementById("password").value;
        let res = await login(uname, passwd);
        if (res.status !== "error") {
            localStorage.setItem("token", res.token);
            localStorage.setItem("refresh_token", res.refresh_token);
            localStorage.setItem("token_type", res.token_type);
            localStorage.setItem("token_exp", res.expiration);
            navigate('/');
        } else {
            document.querySelector("#error").innerText = res.error;
        }
        return false;
    }
} 