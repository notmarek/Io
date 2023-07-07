// TODO: functions to change theme color etc
export const ThemeManager = {
	styleEl: null,
	style: {
		accentColor: "#ff0069",
		backgroundColor: "#121212",
		primaryTextColor: "#ffffff",
		secondaryTextColor: "#a7a7a7",
		primaryLinkColor: "#ffffff",
		secondaryLinkColor: "#5c5c5c",
	},
	_css: null,
	get css() { return this._css },
	set css(val) { this._css = val; this.styleEl.innerHTML = val; },
	init(el = null) {
		for (let prop in this.style) 
			this.style[prop] = localStorage.getItem("theme." + prop) || this.style[prop];
		this.inject(el);
		return this.compile();
	},
	compile() {
		let css = "";
		for (let prop in this.style) {
			const val = this.style[prop];
			prop = "--" + prop.replaceAll(/[A-Z]/g, (m) => "-" + m.toLowerCase());
			css += `${prop}: ${val};\n`;
		}
		this.css = `:root { ${css} }`;
		return css;
	},
	set(prop, val) {
		if (!this.style.hasOwnProperty(prop)) return false;
		this.style[prop] = val;
		localStorage.setItem("theme." + prop, val);
		return this.compile();
	},
	inject(el = null) {
		if (!el) el = document.head;
		this.styleEl = document.createElement("style");
		el.appendChild(this.styleEl);
	},
}
