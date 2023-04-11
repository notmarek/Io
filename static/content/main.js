import { submit } from './api.js'
import { get_info } from './api_client.js';
console.log(await get_info())
let path = window.location.pathname;
window.addEventListener("locationchange", () => { path = window.location.pathname; render(); });
window.addEventListener("popstate", () => { window.dispatchEvent(new Event("locationchange")); console.log("popstae") });
const go_to = (p) => {
    history.pushState({}, document.title, window.location.origin + p);
    window.dispatchEvent(new Event("popstate"));
} 

const render = () => {
    let obj = {
        "/": () => {
            document.body.innerHTML = "kekw nigga";
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself"
        },
        "/user/login": () => {
            fetch("/content/modules/user/login.html").then(r => r.text())
            .then(r => document.body.innerHTML = r)
        }
        
    };
    let fn = obj[path];
    if (fn != undefined)
        fn();
    else {
        console.log("Oh no", path)
        go_to("/");
    }
    
    
}
render();