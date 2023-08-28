import { submit, navigate, token, self, save_tokens_from_response } from "./api.js";
import { get_info, user } from "./api_client.js";
import { ThemeManager } from "./theme.js";
let DEBUG = location.search.includes("?debug") || ["localhost", "127.0.0.1"].includes(window.location.hostname);

window.renderState = {
	currently_rendered_views: {}, // { dom_id: view_name }
	current_view_variables: {},
	render(view, dom_id, vars = {}) {
		renderModule(view, dom_id, vars);
		this.current_view_variables[dom_id] = Object.keys(vars).length === 0 ? null : vars;
		if (!this.is_being_rendered(view, dom_id))
			this.currently_rendered_views[dom_id] = view;
	},
	is_being_rendered(view, dom_id) {
		return this.currently_rendered_views.hasOwnProperty(dom_id) && this.currently_rendered_views[dom_id] === view;
	},
	unrender(dom_id) {
		this.current_view_variables[dom_id] = null;
		this.currently_rendered_views[dom_id] = null;	
		document.querySelector(dom_id).innerHTML = null;
	},
	
}

window.submit = submit;
export let path = window.location.pathname;
window.addEventListener("popstate", () => {
    path = window.location.pathname;
    router();
});
window.onkeydown = (e) => {
    if (e.ctrlKey && e.keyCode === 70 && !renderState.is_being_rendered("search/overlay", "#overlay"))
	{
		e.preventDefault();
		renderState.render("search/overlay", "#overlay");
	}
	if (e.keyCode === 27) {
		renderState.unrender("#overlay");
	}
	return true;
}
window.addEventListener(`click`, async (e) => {
    const origin = e.target.closest(`a`);
	if (origin && origin.href) {
        e.preventDefault();
		if (origin.href.includes((await get_info()).storage)){
			log("LinkDetour", `Attempting to copy '${origin.href}' to the clipboard.`);
			if (navigator.clipboard)
				navigator.clipboard.writeText(origin.href);
			else 
				error("LinkDetour", `Couldn't copy to clipboard, clipboard api not available!`);
			return false;		
		}
		if (renderState.is_being_rendered("search/overlay", "#overlay"))
			renderState.unrender("#overlay")
        navigate(origin.href + (DEBUG ? "?debug" : ""));
        log("LinkDetour", `Soft navigating to ${origin.href}.`);
        return false;
    }
});
const error = (src, msg) => {
	log(`${src}:Error`, msg, "#F00");
}
const log = (src, msg, text_color = null) => {
	return console.log(`%c[${src}] %c${msg}`,  `color: ${window.ThemeManager.style.accentColor}`, `color: ${text_color || window.ThemeManager.style.primaryTextColor }`);
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
        .then(async (r) => {
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
            let p = await self.get_permissions();
			
            if (!p.match(/administrator|_users|_library/)) {
                r = r.replaceAll("%%administrator%%", "hidden");
			}
            
            r = r.replace(/%%/g, "#");
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
                    renderState.render("home/unverified", "#main");
                } else {
                    renderState.render("home/authenticated", "#main");
                }
            } else {
                renderState.render("home/unauthenticated", "#main");
            }
        },
        "/help": () => {
            document.body.innerHTML = "go fuck yourself";
        },
        "/user/login": () => {
            renderState.render("user/login", "#main");
        },
        "/user/logout": () => {
            localStorage.clear();
            navigate("/");
        },
        "/user/register": () => {
            renderState.render("user/register", "#main");
        },
        "/user/settings": () => {
            renderState.render("user/settings", "#main");
        },
		"/admin": () => {
			renderState.render("admin/index", "#main");
		},
        "/admin/library": () => {
            // TODO: don't let everyone in here :)
            renderState.render("admin/library/manage", "#main");
        },
        "/admin/library/create": () => {
            // TODO: don't let everyone in here :)
            renderState.render("admin/library/create", "#main");
        },
		"/admin/user": () => {
			renderState.render("admin/user/all", "#main");
		},
    };
    const smartRoutes = {
        "/library/(?<library_id>.*?)$": ({ library_id }) => {
            renderState.render("library/authenticated", "#main", { library_id });
        },
        "/folder/(?<folder_id>.*?)$": ({ folder_id }) => {
            renderState.render("folder/authenticated", "#main", { folder_id });
        },
		"/admin/user/(?<user_id>.*?)$": ({ user_id }) => {
			renderState.render("admin/user/edit", "#main", { user_id });
		},
    };
    if (token()) renderState.render("header/authenticated", "#header div.buttons");
    else renderState.render("header/unauthenticated", "#header div.buttons");

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
		return renderState.render("misc/404", "#main");
    }
};

window.ThemeManager = ThemeManager;
setup_storage();
if (DEBUG) {
	const { dbgr } = await import("./debugger.js");
	renderModule = (await dbgr(path, self, renderModule, log)).renderModule;
} else {
	console.log = () => null;
	window.showDbgr = () => null;
}
window.ThemeManager.init();
if (localStorage.getItem("refresh_token")) 
	user.refresh_token(localStorage.getItem("refresh_token")).then(res => {	
		if (res.status !== "error")
			save_tokens_from_response(res);
	}
);
router();
