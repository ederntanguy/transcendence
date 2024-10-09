import {
    get_pfp_uri_from_cookie,
    get_tournament_username_from_cookie,
    get_username_from_cookie
} from "../cookie_getter.js";
import { create_button, create_element } from "../lib.js";
import { load_html } from "../HtmlPageGetter.js";
import { set_default_submit, setup_credentials } from "./credentials.js";
import { connexion_close, delete_user } from "./friends_communication.js";
import { show_user_info } from "./statistics.js";

let default_pfp;

const update_info = async () => {
    await load_html('static/account/update_credentials.html', document.querySelector("#app"));
    document.getElementById('username_input').value = get_username_from_cookie();
    document.getElementById('show_pfp').src = default_pfp;
    if (get_tournament_username_from_cookie())
        document.getElementById('tournament_username_input').value = get_tournament_username_from_cookie();
    await setup_credentials('account/update', default_pfp);
}

export const logout_account = () => {
    set_default_submit('account/logout', connexion_close).send();
}

const delete_account = () => {
    delete_user();
    set_default_submit('account/delete', null).send();
}

export const show_account_information = async () => {
    if (!get_username_from_cookie()) {
        create_button('app', 'Sign in', () => navigate_to('/account/signin'));
        create_button('app', 'Sign up', () => navigate_to('/account/signup'));
    } else {
        await load_html('static/account/logged.html', document.querySelector("#app"));
        create_element('h4', 'account_info', get_username_from_cookie(), 'player_username');
        const image = create_element('img', 'profile_picture', '', 'pfp');
        image.src = get_pfp_uri_from_cookie();
        default_pfp = image.src;
        create_button('buttons', 'Stats', show_user_info);
        create_button('buttons', 'Log out', logout_account);
        create_button('buttons', 'Update', update_info);
        create_button('buttons', 'Delete', delete_account);
    }
}
