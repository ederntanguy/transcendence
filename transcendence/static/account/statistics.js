import { get_csrf_token, get_username_from_cookie } from "../cookie_getter.js";
import { load_html } from "../HtmlPageGetter.js";
import { create_element } from "../lib.js";

const GET_INFO_FAILURE = 1;

const show_game_info = (game, id) => {
    const str_duration = game['duration'].toString();
    const game_duration = parseFloat(str_duration.substring(0, 1)) * 3600 + parseFloat(str_duration.substring(2, 4)) * 60 + parseFloat(str_duration.substring(5))
    let local_time = new Date(game['date']);
    local_time = new Date(local_time + ' UTC');
    local_time = local_time.toLocaleString();
    let div = create_element('div', 'all_games', null, id);
    div.classList.add("hist_entry");
    create_element('div', id, null, id+"info").classList.add("info");
    create_element('p', id+"info", local_time);
    create_element('p', id+"info", "Duration: " + Math.floor(game_duration / 60) + 'min ' + parseInt(game_duration) % 60 + 's').classList.add("style_class");
    create_element('p', id+"info", "Opponent: " + game['opponent_username']);
    create_element('div', id, null, id+"summary").classList.add("summary");
    create_element('p', id+"summary", (game['own_score']).toString()).classList.add("scores");
    create_element('p', id+"summary", (game['opponent_score']).toString()).classList.add("scores");
    if (game['winner'] === game['own_username'])
        div.style.backgroundColor = '#198621';
    else
        div.style.backgroundColor = '#842019';
}

const set_statistics = (games_result) => {
    let win = 0;
    let lose = 0;
    let point_win = 0;
    let point_lose = 0;
    let average_party_duration_in_sec = 0;
    for (const i in games_result) {
        const str_duration = games_result[i]['duration'];
        average_party_duration_in_sec += parseFloat(str_duration.substring(0, 1)) * 3600 + parseFloat(str_duration.substring(2, 4)) * 60 + parseFloat(str_duration.substring(5))
        point_win += games_result[i]['own_score'];
        point_lose += games_result[i]['opponent_score']
        show_game_info(games_result[i], i);
        if (games_result[i]['winner'] === get_username_from_cookie())
            win++;
        else
            lose++;
    }
    if (average_party_duration_in_sec > 0)
        average_party_duration_in_sec /= games_result.length;
    document.getElementById('win_ratio').innerText = win + '/' + lose;
    document.getElementById('point_ratio').innerText = point_win + '/' + point_lose;
    if (average_party_duration_in_sec === 0)
        document.getElementById('average_party_duration').innerText = '0min 0s';
    else
        document.getElementById('average_party_duration').innerText = Math.floor(average_party_duration_in_sec / 60) + 'min ' + parseInt(average_party_duration_in_sec) % 60 + 's';
}

const get_info_from_db = (post_url) => {
    let xhr = new XMLHttpRequest();
    xhr.open('POST', post_url, true);
    xhr.setRequestHeader("X-CSRFToken", get_csrf_token());

    xhr.onreadystatechange = async () => {
        if (xhr.readyState !== XMLHttpRequest.DONE)
            return null;
        if (parseInt(xhr.responseText[0]) === GET_INFO_FAILURE) {
            const error_msg = xhr.responseText.substring(1);
            const error_element = document.getElementById('error_msg');
            if (error_element)
                error_element.innerText = error_msg;
            else
                create_element('p', 'app', error_msg, 'error_msg');
        }
        set_statistics(JSON.parse(xhr.responseText));
    };
    xhr.send(null);
}

export const show_user_info = async () => {
    await load_html('static/account/statistics.html', document.querySelector("#app"));
    get_info_from_db('account/statistics', null);
}
