import {get_csrf_token, get_tournament_username_from_cookie, get_username_from_cookie} from "../cookie_getter.js";
import { navigate_to } from "../router.js";
import { create_element } from "../lib.js";
import { setup_friends_socket, update_name } from "./friends_communication.js";

const SIGNING_SUCCESS = '0';
const SIGNING_FAILURE = '1';

const create_from_date = (default_pfp) => {
    let form_data = new FormData();
    const username = get_username_from_cookie();
    if (username) { // when update information
        form_data.append('u', username);
        if (document.getElementById('username_input'))
            form_data.append('nu', document.getElementById('username_input').value);
    }
    else
        form_data.append('u', document.getElementById('username_input').value);
    if (document.getElementById('password_input'))
        form_data.append('p', document.getElementById('password_input').value);
    const new_password = document.getElementById('new_password_input');
    if (new_password)
        form_data.append('np', new_password.value);
    const new_tournament_username = document.getElementById('tournament_username_input');
    if (new_tournament_username)
        form_data.append('ntu', new_tournament_username.value);
    if (document.getElementById('profile_picture')) {
        const pfp = document.getElementById('profile_picture');
        if (pfp && pfp.src !== default_pfp)
            form_data.append('pfp', pfp.files[0]);
    }
    return form_data;
}

export const set_default_submit = (post_url, success_function) => {
    let xhr = new XMLHttpRequest();
    xhr.open('POST', post_url, true);
    xhr.setRequestHeader("X-CSRFToken", get_csrf_token());

    xhr.onreadystatechange = async () => {
        if (xhr.readyState !== XMLHttpRequest.DONE)
            return;
        if (xhr.responseText[0] === SIGNING_SUCCESS) {
            if (success_function)
                success_function()
            await navigate_to('/');
        } else if (xhr.responseText[0] === SIGNING_FAILURE) {
            const error_msg = xhr.responseText.substring(1);
            const error_element = document.getElementById('error_msg');
            if (error_element)
                error_element.innerText = error_msg;
            else
                create_element('p', 'app', error_msg, 'error_msg');
        }
    };
    return xhr;
}

const submit_credentials = (post_url, default_pfp) => {
    if (post_url === 'signup' || post_url === 'signin')
        set_default_submit(post_url, setup_friends_socket).send(create_from_date(default_pfp));
    else
        set_default_submit(post_url, update_name).send(create_from_date(default_pfp));
}

const set_pfp_preview = (event, default_pfp) => {
    const file = event.target.files[0];
    if (file) {
        let reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = function(e) {
            let image_display = document.getElementById('show_pfp');
            image_display.src = e.target.result;
        };
    } else
        document.getElementById('show_pfp').src = default_pfp;
}

export const setup_credentials = (url, default_pfp = null) => {
    if (url === 'signin') {
        document.getElementById("set_profile_picture").remove();
        document.getElementById("profile_picture").remove();
    }
    else
        document
            .getElementById('profile_picture')
            .addEventListener("change", (event) => set_pfp_preview(event, default_pfp));
    document
        .getElementById("password_input")
        .addEventListener(
            "keydown",
            (e) => {
                if (e.key === "Enter")
                    submit_credentials(url, default_pfp);
            },
            {passive: true}
        );
    document
        .getElementById("submit_button")
        .addEventListener("click", () => submit_credentials(url, default_pfp), {passive:true});
}

export const setup_sign_up = async () => setup_credentials("signup", '/static/account/default.png');

export const setup_sign_in = async () => setup_credentials("signin", '/static/account/default.png');
