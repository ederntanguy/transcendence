import { render_elements, tournament_algorithm, tournament_game_result } from "./game_engine.js";
import { l_down, l_up, r_down, r_up, reset_player_direction, set_player_direction } from "./wall.js";
import { create_element, sleep } from "../lib.js";
import { close_connexion, send_information } from "./communication.js";
import { say_game_done_to_friends, say_in_game_to_friends } from "../account/friends_communication.js";
import { determine_bot_pos, reset_tab_pos } from "./bot.js";

const LEFT_WIN = 0
const RIGHT_WIN = 1
const WIN_BY_WITHDRAWAL = 2
const UNEXPECTED_DISCONNECTION = -2
const MODE_MULTi = 0;

let previous_actualise_bot = 0;
let is_multi = -1;
let is_bot_game = false;
let own_side= -1;
let previous_time;
let previous_input = [0, 0];
let win_side = -1;
let left_wall, right_wall;
let ball;
let left_point = 0;
let right_point = 0;
let is_game_aborted = false;
let next_pos_bot = 0.5
let nb_frame_between_bot_render = 0;
let is_tournament = false;
let tournament_players_username = null;

export let is_game_run = false;

export const set_is_tournament_to_true = () => {
    is_tournament = true;
}

export const need_abort_game = () => {
    is_game_aborted = true;
}

const update_position = (value) => {
    if (is_bot_game) {
        nb_frame_between_bot_render++;
        if (previous_actualise_bot + 1000 <= Date.now()) {
            const tmp = nb_frame_between_bot_render;
            nb_frame_between_bot_render = 0;
            previous_actualise_bot = Date.now();
            next_pos_bot = determine_bot_pos([value[2], value[3]], tmp);
        }
    }
    left_wall.y = value[0];
    right_wall.y = value[1];
    ball.x = value[2];
    ball.y = value[3];
}

const update_score = (value) => {
    if (is_bot_game) {
        reset_tab_pos();
        previous_actualise_bot = Date.now() - 850;
        next_pos_bot = 0.5;
        nb_frame_between_bot_render = -1;
    }

    const side_point = value.shift();
    if (side_point === 0)
        left_point++;
    else if (side_point === 1)
        right_point++;
    else
        win_side = close_connexion();
    update_position(value);
}

const update_game_done = (side) => {
    if (side[0] === 0) {
        win_side = LEFT_WIN;
        left_point++;
    }
    else if (side[0] === 1) {
        win_side = RIGHT_WIN;
        right_point++;
    }
    else
        win_side = close_connexion();
}

export const update_value_manager = (value) => {
    if (value === null) {
        win_side = close_connexion();
        return;
    }
    const msg_id = value.shift();
    if (msg_id === 0)
        update_position(value);
    else if (msg_id === 1)
        update_score(value);
    else if (msg_id === 2)
        update_game_done(value);
    else if (msg_id === 3)
        win_side = WIN_BY_WITHDRAWAL;
    else
        win_side = close_connexion();
}

const send_input = (previous_input) => {
    if (is_multi === MODE_MULTi) {
        const next_input = [r_up === 1 ? r_up : l_up, r_down === 1 ? r_down : l_down];
        if (!(previous_input[0] === next_input[0] && previous_input[1] === next_input[1])) {
            send_information([next_input[1] - next_input[0]]);
        }
        return next_input;
    } else {
        let next_input = [l_down - l_up, r_down - r_up];

        if (is_bot_game) {
            next_input = [0, 0];
            next_input[0] = (r_down === 1 ? r_down : l_down) - (r_up === 1 ? r_up : l_up);
            if (next_pos_bot === null || Math.abs(- next_pos_bot + right_wall.y + 0.05) < 0.025)
                next_input[1] = 0;
            else if (0 > - next_pos_bot + right_wall.y + 0.05)
                next_input[1] = 1;
            else
                next_input[1] = -1;
        }
        if (!(previous_input[0] === next_input[0] && previous_input[1] === next_input[1])) {
            send_information([next_input[0], next_input[1]]);
        }
        return next_input;
    }
}

export const game_loop = async (time) => {
    previous_input = send_input(previous_input);
    render_elements(left_wall, right_wall, ball, left_point, right_point);
    if (win_side === -1 && !is_game_aborted) {
        window.requestAnimationFrame(game_loop);
    } else {
        is_game_run = false;
        say_game_done_to_friends();
        is_game_aborted = false;
        is_bot_game = false;
        window.removeEventListener('keydown', set_player_direction, {capture:true});
        window.removeEventListener('keyup', reset_player_direction, {capture:true});
        create_element('div', 'app', null, 'la_kaka');
        let p = create_element('p', 'la_kaka');
        p.classList.add("victory_message"); //add css to losing side and error
        if (win_side === UNEXPECTED_DISCONNECTION)
            p.innerText = 'server error';
        else if (win_side === LEFT_WIN || win_side === RIGHT_WIN) {
            if (is_multi === MODE_MULTi)
                p.innerText = win_side === own_side ? 'Congratulation You Win !' : 'Looser.';
            else {
                if (is_tournament) {
                    p.innerText = win_side === LEFT_WIN ? tournament_players_username[0] + ' win !' : tournament_players_username[1] + ' win !';
                    tournament_game_result(tournament_players_username[0], tournament_players_username[1],
                        win_side === LEFT_WIN ? tournament_players_username[0] : tournament_players_username[1]);
                    is_tournament = false;
                    await sleep(2000);
                    await tournament_algorithm();
                }
                else
                    p.innerText = win_side === LEFT_WIN ? 'Left Win !' : 'Right Win !';

            }
        }
        else if (win_side === WIN_BY_WITHDRAWAL)
            p.innerText = 'Congratulation, you win by withdrawal of your opponent !';
    }
    previous_time = time;
}

export const launch_game = async () => {
    reset_tab_pos();
    previous_time = performance.now();
    previous_actualise_bot = Date.now() + 800;
    await game_loop(performance.now());
}

export const init_game_info = async (game_mode, value, element, is_PvE, get_tournament_players_username) => {
    tournament_players_username = get_tournament_players_username;
    if (is_PvE)
        is_bot_game = true
    is_game_run = true;
    say_in_game_to_friends();
    is_game_aborted = false;
    win_side = -1;
    left_point = 0;
    right_point = 0;
    left_wall = element[0];
    right_wall = element[1];
    ball = element[2];
    is_multi = game_mode;
    if (game_mode === MODE_MULTi) {
        create_element('p', 'app', "opponent: " + value[0], 'players_username');
        own_side = value[1];
        return value[2];
    }
    else if (tournament_players_username != null) {
        create_element('p', 'app', tournament_players_username[0] + " VS " + tournament_players_username[1], 'players_username');
    }
    return value[1];
}
