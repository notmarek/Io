import { submit, navigate, token, self } from './api.js'
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



let renderModule = (path, dom_id) => {
    let el = document.querySelector(dom_id);
    fetch(`/content/modules/${path}`).then(r => r.text())
    .then(r => el.innerHTML = r).then(_ => {
        for (let script of el.querySelectorAll("script")) {
            const blob = new Blob([script.innerText], {
                type: "application/javascript",
            });
            let uri = URL.createObjectURL(blob);
            let module = import(uri);
            
            module.then(e=>e.run());
            let fn = (script.innerText);
            console.log("this is ", this)
            fn.call(execution_context_hack);
        }
    });
}

const render = () => {
    
    let obj = {
        "/": async () => {
            if (token()) {
                if (!(await self.get_permissions()).includes("verified")){
                    renderModule("home/unverified.html", "#main")
                } else {
                    renderModule('home/authenticated.html', "#main");
                }
            } else {
                renderModule('home/unauthenticated.html', "#main");
            }
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself"
        },
        "/user/login": () => {
            renderModule("user/login.html", "#main")
        },
        "/user/logout": () => {
            localStorage.clear();
            navigate("/");
        },
        "/user/register": () => {
            renderModule("user/register.html", "#main")
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
        console.log(path);
        renderModule(path.slice(1), "#main");
        // console.log("Oh no", path)
        // navigate("/");
    }
    
    
}
render();