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

const dbgr = async () => {
    let container = document.createElement("div");
    window.showDbgr = () => container.hidden = false;
    container.onclick = (e) => container.hidden = 1;
    container.style = "z-index:99999999; position: absolute; top:0;left:0;background: #000000af;padding:5px;";
    container.innerHTML = `Endpoint: ${location.host}<br>
    Module: <span id="dbgr-path">${path}</span><br>
    Username: ${await self.get_username()}<br>
    Permissions: ${await self.get_permissions()}`;

    document.body.appendChild(container)
}   


let renderModule = (path, dom_id) => {
    document.querySelector("#dbgr-path").innerText = path;
    let el = document.querySelector(dom_id);
    fetch(`/content/modules/${path}`).then(r => r.text())
    .then(r => el.innerHTML = r).then(_ => {
        for (let script of el.querySelectorAll("script")) {
            let mutated = script.innerText.replace("from \"", `from "${location.origin}`);
            mutated = mutated.replace("console.log", `console.log.bind(console, "%c[Modules/${path}]", "color: #ff0069")`);
            const blob = new Blob([mutated], {
                type: "application/javascript",
            });
            let uri = URL.createObjectURL(blob);
            let module = import(uri);
            
            module.then(e=>e.run());
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
        // if  \/library\/(.*?)/
        console.log(path);
        renderModule(path.slice(1), "#main");
        // console.log("Oh no", path)
        // navigate("/");
    }
    
    
}
await dbgr();
render();