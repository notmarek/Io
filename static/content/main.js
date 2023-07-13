import { submit, navigate, token, self, save_tokens_from_response } from "./api.js";
import { get_info, user } from "./api_client.js";
import { ThemeManager } from "./theme.js";
let DEBUG = true;

window.submit = submit;
export let path = window.location.pathname;
window.addEventListener("popstate", () => {
    path = window.location.pathname;
    router();
});
window.addEventListener(`click`, (e) => {
    const origin = e.target.closest(`a`);
    if (origin && origin.href) {
        e.preventDefault();
        navigate(origin.href);
        log("Navigation", `Soft navigating to ${origin.href}.`);
        return false;
    }
});
const log = (src, msg) => {
	return console.log.bind(console, `%c[${src}]`, `color: ${window.ThemeManager.style.accentColor}`)(msg);
}

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

export let renderModule = (module_path, dom_id, variables = null) => {
    module_path = module_path + ".html";
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
    if (fn != undefined) {
		log("Router", `Found a dumb route for '${path}'`);
		return fn()
	}
    else {
        // if  \/library\/(.*?)/
        for (const [matcher, fn] of Object.entries(smartRoutes)) {
            let match = path.match(matcher);
            if (match) {	
				log("Router", `Found a smart route for '${match}'`);
            	return fn.apply(undefined, [match.groups]);
			}
        }
		log("Router", `No route found for '${path}' returning 404`);
		return renderModule("misc/404", "#main");
    }
};

const setup_theme_manager = () => {
	window.ThemeManager = ThemeManager;
	window.ThemeManager.init();
};
setup_theme_manager();
setup_storage();
if (DEBUG) {
	const { dbgr } = await import("./debugger.js");
	renderModule = (await dbgr(path, self, renderModule, log)).renderModule;
} else {
	console.log = { bind(...a){return (...a) => {}}};
}
if (localStorage.getItem("refresh_token")) 
	user.refresh_token(localStorage.getItem("refresh_token")).then(res => {	
		if (res.status !== "error")
			save_tokens_from_response(res);
	}
);
router();
