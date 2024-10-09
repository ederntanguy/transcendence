import { show_notifs, state_of_friends_list, toggle_friends_popup } from "../base.js";
import {get_username_from_cookie} from "../cookie_getter.js";

export let friends_socket = null;
export let friends_list = null
export let friends_request_send = null
export let friends_request_received = null

export const connexion_close = () => {
    if (friends_socket)
        friends_socket.close();
    friends_socket = null;
    friends_list = null;
    friends_request_send = null;
    friends_request_received = null;
    show_friends_list();
    if (document.getElementById('friends_popup').style.display === 'flex')
        toggle_friends_popup()
}

export const update_name = () => {
    friends_socket.send(JSON.stringify(['update_username', get_username_from_cookie()]));
}

export const delete_user = () => {
    friends_socket.send(JSON.stringify(['delete_user']));
    connexion_close();
}

export const remove_friend = (friend_username) => {
    friends_socket.send(JSON.stringify(['remove_friend', friend_username]));
}

export const cancel_friend_request = (friend_username) => {
    friends_socket.send(JSON.stringify(['cancel_friend_request', friend_username]));
}

export const accept_friend_request = (friend_username) => {
    friends_socket.send(JSON.stringify(['accept_friend_request', friend_username]));
}

export const refuse_friend_request = (friend_username) => {
    friends_socket.send(JSON.stringify(['refuse_friend_request', friend_username]));
}

export const say_in_game_to_friends = () => {
    if (friends_socket)
        friends_socket.send(JSON.stringify(['in game']));
}

export const say_game_done_to_friends = () => {
    if (friends_socket)
        friends_socket.send(JSON.stringify(['game done']));
}

export const setup_friends_socket = () => {
    friends_socket = new WebSocket('wss://' + window.location.host + '/ws/friends/' + get_username_from_cookie() + '/');
    friends_socket.onopen = () => {
        friends_socket.send(JSON.stringify(['hello']));
        if (document.getElementById('friends_popup').style.display === 'flex')
            toggle_friends_popup();
    };
    friends_socket.onmessage = (e) => {
        const data = JSON.parse(e.data)
        if (data.friends_list !== undefined)
            friends_list = data.friends_list;
        if (data.friends_request_received !== undefined)
            friends_request_received = data.friends_request_received;
        if (data.friends_request_send !== undefined)
            friends_request_send = data.friends_request_send;
        if (state_of_friends_list === 1)
            show_friends_list();
        else if (state_of_friends_list === 2)
            show_notifs();
    };
    friends_socket.onclose = () => {
        connexion_close();
    };
}
