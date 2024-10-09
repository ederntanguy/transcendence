export const load_html = (path, element) => {
	return fetch(path)
		.then((response) => response.text())
		.then((html_text) => {
			element.innerHTML = html_text;
		});
}

const load_script = (js_url) => {
	return new Promise((resolve, reject) => {
		const app_div = document.querySelector('#app');
		let script = document.createElement('script');
		script.addEventListener('load', resolve);
		script.addEventListener('error', () => reject(`Failed to load script ${js_url}`));
		script.type = 'module';
		script.async = true;
    	script.src = js_url;
		app_div.appendChild(script);
	});
}

const load_resources = (element, html_path, script_path) => {
	const html_loaded_promise = load_html(html_path, element);
	if (script_path === null)
		return html_loaded_promise;
	const script_loaded_promise = load_script(script_path);
	return Promise.all([html_loaded_promise, script_loaded_promise]);
}

export default class {
    constructor(html_path, script_path = null, on_page_loaded = null) {
        this.html_path = html_path;
		this.script_path = script_path;
		this.on_page_loaded = on_page_loaded;
    }

    async load_html_into(element) {
		await load_resources(element, this.html_path, this.script_path);
		if (this.on_page_loaded !== null)
			await this.on_page_loaded();
    }
}
