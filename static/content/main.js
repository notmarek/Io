import { submit, navigate, token, self, save_tokens_from_response } from "./api.js";
import { get_info, user } from "./api_client.js";
import { ThemeManager } from "./theme.js";
window.submit = submit;
let path = window.location.pathname;
window.addEventListener("popstate", () => {
    path = window.location.pathname;
    router();
});
window.addEventListener(`click`, (e) => {
    const origin = e.target.closest(`a`);
    if (origin && origin.href) {
        e.preventDefault();
        navigate(origin.href);
        console.log(`Soft navigating to ${origin.href}.`);
        return false;
    }
});
const log = (src, msg) => {
	return console.log.bind(console, `%c[${src}]`, `color: ${window.ThemeManager.style.accentColor}`)(msg);
}
const dbgr = async () => {
    let container = document.createElement("div");

    container.style =
        "z-index:99999999; position: absolute; top:0;left:0;background: #000000af;padding:5px;";
    container.innerHTML = `[Server] host: ${location.host}
    <br>[User] username: ${await self.get_username()}
    permissions: ${await self.get_permissions()}
    <br>[Modules] main: <span id="dbgr-path">${path}</span>
    rendered: <span id="dbgr-num-modules">null</span>
    <br>[Cache] objects: <span id="dbgr-cache-size">null</span>
    hits: <span id="dbgr-cache-hits">null</span>
    misses: <span id="dbgr-cache-misses">null</span>
    invalid: <span id="dbgr-cache-inv">0</span>`;
    let set_og = window.session.set;
    let cache_hits = 0;
    let cache_misses = 0;
    window.session.set = (k, v) => {
        let r = set_og(k, v);
		log("session.set", `Setting '${k}' to '${v}'`);
        document.getElementById("dbgr-cache-size").innerHTML =
            Object.keys(window.session).length - 2;
		document.getElementById("dbgr-cache-inv").innerHTML = window.session.invalid.length;
        return r;
    };
    let invalidate_og = window.session.invalidate;
	window.session.invalidate = (k) => {
		log("session.invalidate", `Invalidated '${k}'`);
		let r = invalidate_og(k);
		document.getElementById("dbgr-cache-inv").innerHTML = window.session.invalid.length;
		return r;
	}
    let get_og = window.session.get;
    window.session.get = (k) => {
        let og = get_og(k);
		log("session.get", `Cache ${og ? "hit" : "miss"} on key "${k}".`); 
        if (og) {
            cache_hits++;
        } else {
            cache_misses++;
        }
        document.getElementById("dbgr-cache-hits").innerHTML = cache_hits;
        document.getElementById("dbgr-cache-misses").innerHTML = cache_misses;
        return og;
    };
	let setItem_og = window.localStorage.setItem;
	localStorage.__proto__.setItem = (...args) => {
		log("localStorage.setItem", `Set '${args[0]}' to '${args[1]}'`)
		return setItem_og.apply(localStorage, args);
	}

    const modules_num = () => {
        if (container.hidden) return;
        document.getElementById("dbgr-num-modules").innerHTML =
            document.querySelectorAll("[module]").length;
    };
    let interval = setInterval(modules_num, 300);
    container.onclick = (e) => (
        (log("dbgr", "Hiding debugger")), (container.hidden = 1), clearInterval(interval)
    );
    window.showDbgr = () => (
        (log("dbgr", "Showing debugger")), (container.hidden = false), (interval = setInterval(modules_num, 300))
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
	invalid: [],
        set: (k, v) => {
	    if (window.session.invalid.includes(k))
		window.session.invalid.splice(window.session.invalid.indexOf(k), 1);
            window.session[k] = v;
            return v;
        },
        get: (k) => {
		if (window.session.invalid.includes(k))
			return undefined;
            return window.session[k];
        },
	invalidate: (k) => {
		window.session.invalid.push(k);
		return true;
	}
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

let renderModule = (module_path, dom_id, variables = null) => {
    module_path = module_path + ".html";
    document.querySelector("#dbgr-path").innerText = module_path;
    let hash = hashCode(module_path).toString(16).replace("-", "_");
    let el = document.querySelector(dom_id);
    get_module(module_path)
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
	    r = r.replace(
                /#(.*?)( |.|,|\))/g,
                `#_${hash}_$1$2`
            );
            r = r.replace(/%%/g, "#");
            self.get_permissions().then((p) => {
                if (!p.includes("administrator"))
                    r = r.replaceAll("%%administrator%%", "hidden");
            });
            return r;
        })
        .then((r) => (el.innerHTML = r))
        .then((_) => {
            for (let script of el.querySelectorAll("script")) {
                let mutated = script.innerText.replaceAll(
                    'from "',
                    `from "${location.origin}`
                );
                mutated = mutated.replace(
                    "console.log",
                    `console.log.bind(console, "%c[Modules/${module_path}]", "color: ${window.ThemeManager.style.accentColor}")`
                );
                const blob = new Blob([mutated], {
                    type: "application/javascript",
                });
                let uri = URL.createObjectURL(blob);
                let js = import(uri);

                js.then((e) => e.run());
            }
        });
};

const router = () => {
    let simpleRoutes = {
        "/": async () => {
            if (token()) {
                if (!(await self.get_permissions()).includes("verified")) {
                    renderModule("home/unverified", "#main");
                } else {
                    renderModule("home/authenticated", "#main");
                }
            } else {
                renderModule("home/unauthenticated", "#main");
            }
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself";
        },
        "/user/login": () => {
            renderModule("user/login", "#main");
        },
        "/user/logout": () => {
            localStorage.clear();
            navigate("/");
        },
        "/user/register": () => {
            renderModule("user/register", "#main");
        },
        "/user/settings": () => {
            renderModule("user/settings", "#main");
        },
        "/admin/library": () => {
            // TODO: don't let everyone in here :)
            renderModule("admin/library/manage", "#main");
        },
        "/admin/library/create": () => {
            // TODO: don't let everyone in here :)
            renderModule("admin/library/create", "#main")
        }
    };
    const smartRoutes = {
        "/library/(?<library_id>.*?)$": ({ library_id }) => {
            renderModule("library/authenticated", "#main", { library_id });
        },
        "/folder/(?<folder_id>.*?)$": ({ folder_id }) => {
            renderModule("folder/authenticated", "#main", { folder_id });
        },
    };
    if (token()) renderModule("header/authenticated", "#header div.buttons");
    else renderModule("header/unauthenticated", "#header div.buttons");

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

const setup_theme_manager = () => {
	window.ThemeManager = ThemeManager;
	window.ThemeManager.init();
};
setup_theme_manager();
setup_storage();
await dbgr();
if (localStorage.getItem("refresh_token")) 
	user.refresh_token(localStorage.getItem("refresh_token")).then(res => {	
		if (res.status !== "error")
			save_tokens_from_response(res);
	}
);
router();
