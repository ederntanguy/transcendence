import {create_button, create_element, create_image} from "./lib.js";
import {
    accept_friend_request, cancel_friend_request,
    friends_list,
    friends_request_received,
    friends_request_send, refuse_friend_request, remove_friend,
} from "./account/friends_communication.js";
import { get_username_from_cookie } from "./cookie_getter.js";

export let state_of_friends_list = 1;

document.addEventListener('DOMContentLoaded', function () {
    const container = document.getElementById('container');
    const movingSquare = document.getElementById('movingSquare');

    let speedX = 0.5; // Speed of the square along the X-axis
    let speedY = 0.3; // Speed of the square along the Y-axis
    let positionX = 0; // Initial X position
    let positionY = 0; // Initial Y position

    function animate() {
        positionX += speedX;
        positionY += speedY;

        if (positionX + movingSquare.offsetWidth > container.offsetWidth - 32 || positionX < 0) {
            speedX *= -1;
        }
        if (positionY + movingSquare.offsetHeight > container.offsetHeight - 32 || positionY < 0) {
            speedY *= -1;
        }

        movingSquare.style.left = positionX + 'px';
        movingSquare.style.top = positionY + 'px';

        requestAnimationFrame(animate);
    }
    animate();
});

const style_status = (element, status) => {
    element.style.height = '20px'
    element.style.width = '20px'
    element.style.borderRadius = '20px'
    if (status === 'connected')
        element.style.background = 'green'
    else if (status === 'disconnected')
        element.style.background = 'grey'
    else
        element.style.background = 'red'
    element.classList.add("friend_status")
}

export const show_friends_list = () => {
    state_of_friends_list = 1;
    const friends_list_show = document.getElementById("friends_list_show");
    document.getElementById("notifs").classList.remove("active");
    document.getElementById("friends_list").classList.add("active");
    friends_list_show.innerHTML = '';
    for (const friend_username in friends_list) {
        if (!document.getElementById(friends_list[friend_username][0]))
            create_element('div', 'friends_list_show', null, friends_list[friend_username][0]).classList.add("friend")
        style_status(create_element('div', friends_list[friend_username][0], null, friends_list[friend_username][0] + '_status'), friends_list[friend_username][1]);
        create_element('div', friends_list[friend_username][0], friends_list[friend_username][0], friends_list[friend_username][0] + '_child');
        create_button(friends_list[friend_username][0], null, ()=> remove_friend(friends_list[friend_username][0]), friends_list[friend_username][0] + '_cross');
        create_image(friends_list[friend_username][0] + '_cross', null, '/static/assets/pixel_x.svg', "cross", null).classList.add("cross");
    }
}

export const show_notifs = () => {
    state_of_friends_list = 2;
    const friends_list_show = document.getElementById("friends_list_show");
    document.getElementById("friends_list").classList.remove("active");
    document.getElementById("notifs").classList.add("active");
    friends_list_show.innerHTML = '';
    for (const friend_username in friends_request_received) {
        if (!document.getElementById(friends_request_received[friend_username]))
            create_element('div', 'friends_list_show', null, friends_request_received[friend_username])
        create_element('div', friends_request_received[friend_username], friends_request_received[friend_username], friends_request_received[friend_username] + '_child').classList.add("friend");
        create_button(friends_request_received[friend_username] + '_child', null, ()=> accept_friend_request(friends_request_received[friend_username]), friends_request_received[friend_username] + '_check');
        create_image(friends_request_received[friend_username] + '_check', null, '/static/assets/pixel_check.svg', "cross", null).classList.add("cross");
        create_button(friends_request_received[friend_username] + '_child', null, ()=> refuse_friend_request(friends_request_received[friend_username]), friends_request_received[friend_username] + '_cross');
        create_image(friends_request_received[friend_username] + '_cross', null, '/static/assets/pixel_x.svg', "cross", null).classList.add("cross");
    }
    for (const friend_username in friends_request_send) {
        if (!document.getElementById(friends_request_send[friend_username]))
            create_element('div', 'friends_list_show', null, friends_request_send[friend_username])
        create_element('div', friends_request_send[friend_username], friends_request_send[friend_username], friends_request_send[friend_username] + '_child').classList.add("friend");
        create_button(friends_request_send[friend_username] + '_child', null, ()=> cancel_friend_request(friends_request_send[friend_username]), friends_request_send[friend_username] + '_cross');
        create_image(friends_request_send[friend_username] + '_cross', null, '/static/assets/pixel_x.svg', "cross", null).classList.add("cross");
    }
}

export const toggle_friends_popup = () => {
    const friends = document.getElementById("friends");
    const popup = document.getElementById("friends_popup");
    const search_input = document.getElementById("add_a_friend");
    const friends_list_type = document.getElementById("friends_list_type");
    if (!get_username_from_cookie()) {
        if (!document.getElementById('no_logining_in_friends'))
            create_element('p', 'friends_list_show', 'You are not signed up.', 'no_logining_in_friends')
        search_input.style.display = "none"
        friends_list_type.style.display = "none"
    } else {
        friends_list_type.style.display = "flex"
        search_input.style.display = "block"
    }
    popup.style.display = popup.style.display === "flex" ? "none" : "flex";
    friends.classList.toggle("release");
}

export const toggle_friends_style = () => {
}

window.toggle_friends_style = toggle_friends_style;
window.toggle_friends_popup = toggle_friends_popup;
window.show_friends_list = () => show_friends_list();
window.show_notifs = () => show_notifs();