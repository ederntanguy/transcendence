import HtmlPageGetter from "./HtmlPageGetter.js";
import { show_account_information } from "./account/account.js";
import { setup_friends_socket, friends_socket } from './account/friends_communication.js'
import { setup_sign_up, setup_sign_in } from './account/credentials.js'
import { choose_mode, handle_running_game } from "./game/game_engine.js";
import { get_username_from_cookie } from "./cookie_getter.js";

const path_to_regex = (path) => new RegExp("^" + path.replace(/\//g, "\\/").replace(/:\w+/g, "(.+)") + "$");

const router = async () => {
    const routes = [
        { path: "/", view: new HtmlPageGetter('static/home_page.html' ) },
        { path: "/chat", view: new HtmlPageGetter('static/chat.html') },
        { path: "/friends", view: new HtmlPageGetter('static/friends.html') },
        { path: "/play", view: new HtmlPageGetter('static/empty.html', 'static/game/game_engine.js', choose_mode) },
        { path: "/account", view: new HtmlPageGetter('static/account/empty.html', 'static/account/account.js', show_account_information) },
        { path: "/account/signup", view: new HtmlPageGetter('../static/account/credentials.html', '../static/account/credentials.js', setup_sign_up) },
        { path: "/account/signin", view: new HtmlPageGetter('../static/account/credentials.html', '../static/account/credentials.js', setup_sign_in) }
    ];
    let route = routes.find((route) => location.pathname.match(path_to_regex(route.path)) !== null);
    if (!route)
        route = routes[0];
    await route.view.load_html_into(document.querySelector("#app"));
}

let last_url = ""

export const navigate_to = async (url) => {
    if (url !== '/play')
        handle_running_game();
    if (!friends_socket && get_username_from_cookie() !== null)
        setup_friends_socket();
    if (last_url !== url)
        history.pushState({path: url}, null, url);
    last_url = url;
    await router();
};

window.addEventListener("popstate", router);

document.addEventListener("DOMContentLoaded", () => navigate_to(location.pathname));

document.getElementById('add_a_friend').addEventListener("keydown", (event) => {
    if (event.key === 'Enter')
        friends_socket.send(JSON.stringify(['request_new_friend', document.getElementById('add_a_friend').value]));
});

window.navigate_to = navigate_to;
