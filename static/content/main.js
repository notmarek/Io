import { submit, navigate } from './api.js'
import { get_info } from './api_client.js';
console.log(await get_info());
window.submit = submit;
let path = window.location.pathname;
window.addEventListener("popstate", () => { path = window.location.pathname; render(); });
window.addEventListener(`click`, e => {
    const origin = e.target.closest(`a`);
    if (origin) {
      e.preventDefault();
      navigate(origin.href);  
      console.log(`Soft navigating to ${origin.href}.`);
      return false;
    }
});


let token = () => {
    return localStorage.getItem("token");
}

let renderModule = (path, dom_id) => {
    fetch(`/content/modules/${path}`).then(r => r.text())
    .then(r => document.querySelector(dom_id).innerHTML = r)
}

const render = () => {
    
    let obj = {
        "/": () => {
            if (token()) {
                renderModule('home/authenticated.html', "#main");
            } else {
                renderModule('home/unauthenticated.html', "#main");
            }
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself"
        },
        "/user/login": () => {
            renderModule("user/login.html", "#main")
        }
        
    };

    if (token())
        renderModule("header/authenticated.html", "#header div.buttons");
    else
        renderModule("header/unauthenticated.html", "#header div.buttons");

    let fn = obj[path];
    if (fn != undefined)
        fn();
    else {
        console.log("Oh no", path)
        navigate("/");
    }
    
    
}
render();