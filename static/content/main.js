import { submit, navigate, token, self } from "./api.js";
import { get_info } from "./api_client.js";
console.log(await get_info());
window.submit = submit;
let path = window.location.pathname;
window.addEventListener("popstate", () => {
    path = window.location.pathname;
    router();
});
window.addEventListener(`click`, (e) => {
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

    container.style =
        "z-index:99999999; position: absolute; top:0;left:0;background: #000000af;padding:5px;";
    container.innerHTML = `Endpoint: ${location.host}<br>
    Module: <span id="dbgr-path">${path}</span><br>
    Username: ${await self.get_username()}<br>
    Permissions: ${await self.get_permissions()}<br>
    Loaded modules: <span id="dbgr-num-modules">null</span>`;
    const modules_num = () => {
        if (container.hidden) return;
        document.getElementById("dbgr-num-modules").innerHTML =
            document.querySelectorAll("[module]").length;
    };
    let interval = setInterval(modules_num, 300);
    container.onclick = (e) => (
        (container.hidden = 1), clearInterval(interval)
    );
    window.showDbgr = () => (
        (container.hidden = false), (interval = setInterval(modules_num, 300))
    );
    document.body.appendChild(container);
};

/**
 * Returns a hash code from a string
 * @param  {String} str The string to hash.
 * @return {Number}    A 32bit integer
 * @see http://werxltd.com/wp/2010/05/13/javascript-implementation-of-javas-string-hashcode-method/
 */
function hashCode(str) {
    let hash = 0;
    for (let i = 0, len = str.length; i < len; i++) {
        let chr = str.charCodeAt(i);
        hash = (hash << 5) - hash + chr;
        hash |= 0; // Convert to 32bit integer
    }
    return hash;
}

const setup_storage = () => {
    window.session = {
        set: (k, v) => {
            window.session[k] = v;
            return v;
        },
        get: (k) => {
            return window.session[k];
        },
    };
};

let get_module = async (path) => {
    return Promise.resolve(
        window.session.get(`/content/modules/${path}`) ||
            (await fetch(`/content/modules/${path}`).then((r) =>
                window.session.set(`/content/modules/${path}`, r.text())
            ))
    );
};

let renderModule = (path, dom_id, variables = null) => {
    document.querySelector("#dbgr-path").innerText = path;
    let hash = hashCode(path).toString(16).replace("-", "_");
    let el = document.querySelector(dom_id);
    get_module(path)
        .then((r) => {
            // Templating like variable injection
            if (variables) {
                for (let [key, value] of Object.entries(variables)) {
                    r = r.replaceAll(`##${key}##`, value);
                }
            }
            return r;
        })
        .then((r) => {
            r = r.replaceAll(/id="(.*?)"/g, `id="_${hash}_$1"`);
            r = r.replaceAll(/idg=/g, `id=`);
            r = r.replaceAll(
                /getElementById\("(.*?)"\)/g,
                `getElementById("_${hash}_$1")`
            );
            r = r.replaceAll("getElementByGId", "getElementById");
            return r;
        })
        .then((r) => (el.innerHTML = r))
        .then((_) => {
            for (let script of el.querySelectorAll("script")) {
                let mutated = script.innerText.replace(
                    'from "',
                    `from "${location.origin}`
                );
                mutated = mutated.replace(/#(.*?)( |.|,|\))/g, `#_${hash}_$1$2`);
                mutated = mutated.replace(/%%/g, "#");

                mutated = mutated.replace(
                    "console.log",
                    `console.log.bind(console, "%c[Modules/${path}]", "color: #ff0069")`
                );
                const blob = new Blob([mutated], {
                    type: "application/javascript",
                });
                let uri = URL.createObjectURL(blob);
                let module = import(uri);

                module.then((e) => e.run());
            }
        });
};

const router = () => {
    let simpleRoutes = {
        "/": async () => {
            if (token()) {
                if (!(await self.get_permissions()).includes("verified")) {
                    renderModule("home/unverified.html", "#main");
                } else {
                    renderModule("home/authenticated.html", "#main");
                }
            } else {
                renderModule("home/unauthenticated.html", "#main");
            }
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself";
        },
        "/user/login": () => {
            renderModule("user/login.html", "#main");
        },
        "/user/logout": () => {
            localStorage.clear();
            navigate("/");
        },
        "/user/register": () => {
            renderModule("user/register.html", "#main");
        },
        "/user/settings": () => {
            renderModule("user/settings.html", "#main");
        },
    };
    const smartRoutes = {
        "/library/(?<library_id>.*?)$": ({ library_id }) => {
            renderModule("library/authenticated.html", "#main", { library_id });
        },
        "/folder/(?<folder_id>.*?)$": ({ folder_id }) => {
            renderModule("folder/authenticated.html", "#main", { folder_id });
        },
    };
    if (token())
        renderModule("header/authenticated.html", "#header div.buttons");
    else renderModule("header/unauthenticated.html", "#header div.buttons");

    let fn = simpleRoutes[path];
    if (fn != undefined) fn();
    else {
        // if  \/library\/(.*?)/
        console.log(path);
        for (const [matcher, fn] of Object.entries(smartRoutes)) {
            let match = path.match(matcher);
            console.log(match);
            if (match) return fn.apply(undefined, [match.groups]);
        }
        // renderModule(path.slice(1), "#main");
        // console.log("Oh no", path)
        // navigate("/");
    }
};
await dbgr();
setup_storage();
router();
