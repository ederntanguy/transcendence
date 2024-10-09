import {create_button, create_element, delete_app_element, sleep} from "../lib.js";
import { make_wall_structs } from "./wall.js";
import {
    create_ball,
    create_walls,
    make_render_target,
    render_ball,
    render_wall
} from "./rendering.js";
import {
    init_game_info,
    is_game_run,
    launch_game,
    need_abort_game, set_is_tournament_to_true
} from "./game_loop.js";
import {close_connexion, establish_connexion, unexpected_quit} from "./communication.js";
import {get_tournament_username_from_cookie, get_username_from_cookie} from "../cookie_getter.js";
import { navigate_to } from "../router.js";

export let render_target;

export const RATIO = 1.3;
export const BALL_EDGE = 0.017;
/// The ball is square, so that's not really a radius.
export const BALL_RADIUS = BALL_EDGE / 2.0;
export const PAD_WIDTH = 0.015;
export const PAD_HEIGHT = 0.100;

let in_queue = false
let tournament_players_info;
const make_ball_struct = () => {
    return {
        x: 0.5 * RATIO,
        y: 0.5,
    };
}

const delete_btn_in_queue = () => {
    document.getElementById('queue').remove();
    document.getElementById('unregister').remove();
}

const unregister_multi = async () => {
    close_connexion();
    in_queue = false;
    delete_btn_in_queue();
    await choose_mode();
}

export const register_in_queue = () => {
    delete_app_element();
    create_element('div', 'body', 'in queue', 'queue');
    create_button('app', 'unregister', async () => unregister_multi(), 'unregister');
    in_queue = true;
}

export let handle_running_game = () => {
    // implement for local when it's finish and if it keeps
    if (is_game_run === true) {
        need_abort_game();
        unexpected_quit();
    }
}

export const render_elements = (left_wall, right_wall, ball, left_point, right_point) => {
    render_wall(left_wall, render_target, document.getElementById('left_wall'));
    render_wall(right_wall, render_target, document.getElementById('right_wall'));
    render_ball(ball, render_target, document.getElementById('ball'));
    if (document.getElementById('left_score'))
    {
        document.getElementById('left_score').innerHTML = left_point + '';
        document.getElementById('left_score').classList.add("left_score");
    }

    if (document.getElementById('right_score'))
    {
        document.getElementById('right_score').innerHTML = right_point + '';
        document.getElementById('right_score').classList.add("right_score");
    }
}

const starting_screen = async (starting_time, timer) => {
    timer.classList.add("starting_timer")
    let time = Date.now();
    while (time < starting_time) {
        time = Date.now();
        let diff = Number(BigInt(starting_time) - BigInt(time));
        timer.innerHTML = Math.ceil(diff / 1000).toString();
        await sleep(10);
    }
}

const init_element = () => {
    let walls = make_wall_structs();
    let left_wall = walls[0];
    let right_wall = walls[1];
    let ball = make_ball_struct();
    const game_element = document.getElementById('playground');
    create_walls(game_element);
    create_ball(game_element);
    render_elements(left_wall, right_wall, ball, 0, 0);
    return [left_wall, right_wall, ball];
}

export const init_game = async (game_mode, value, is_PvE, tournament_players_username) => {
    let starting_time;
    if (game_mode === 0) {
        if (location.pathname !== '/play')
            await navigate_to('/play');
        in_queue = false;
    }
    delete_app_element()
    create_element('p', 'app', 0 + '', 'left_score');
    create_element('p', 'app', 0 + '', 'right_score');
    render_target = make_render_target();
    let [left_wall, right_wall, ball] = init_element();
    starting_time = await init_game_info(game_mode, value, [left_wall, right_wall, ball], is_PvE, tournament_players_username);
    if (!starting_time)
        starting_time = Date.now() + 4700;
    await starting_screen(starting_time, create_element('p', 'app', null, 'start_timer'));
    if (document.getElementById('start_timer'))
        document.getElementById('start_timer').remove();
    await launch_game();
}

const select_solo_mode = () => {
    delete_app_element();
    create_button('app', 'PvP', () => establish_connexion(1), 'PvP_button').classList.add("play_button");
    create_button('app', 'PvE', () => establish_connexion(1, true), 'PvE_button').classList.add("play_button");
}

const tournament_game_announce = (player1, player2) => {
    delete_app_element();
    create_element("div", "app", "", "match_announcement").classList.add("fait_toi_plaiz");
    create_element("p", "match_announcement", player1, null);
    create_element("p", "match_announcement", "VS", null);
    create_element("p", "match_announcement", player2, null);
}

export const tournament_game_result = (username_player1, username_player2, username_winner) => {
    for (let player of tournament_players_info) {
        if (player[0] === username_player1)
            player[2]++;
        else if (player[0] === username_player2)
            player[2]++;
        if (player[0] === username_winner)
            player[1]++;
    }
}

const determine_who_play = () => {
    let player1= null;
    let player2 = null;
    let nb_game_played = 0;
    for (const player of tournament_players_info)
        nb_game_played += player[2];
    if (nb_game_played === 0)
        return [tournament_players_info[0], tournament_players_info[1]];
    nb_game_played /= 2;
    if (nb_game_played === 1)
        return [tournament_players_info[2], tournament_players_info[3]];
    else if (nb_game_played === 2) {
        for (const player of tournament_players_info) {
            if (player[1] === 0 && player1 === null)
                player1 = player;
            else if (player[1] === 0)
                player2 = player;
        }
    } else if (nb_game_played === 3) {
        for (const player of tournament_players_info) {
            if (player[1] === 1 && player[2] === 1 && player1 === null)
                player1 = player;
            else if (player[1] === 1 && player[2] === 1)
                player2 = player;
        }
    }
    return [player1, player2];
}

export const tournament_algorithm = async () => {
    show_tournament_score();
    await sleep(2000);
    let player1;
    let player2;
    [player1, player2] = determine_who_play();
    if (player1 != null && player2 != null) {
        tournament_game_announce(player1[0], player2[0]);
        await sleep(2000);
        set_is_tournament_to_true();
        establish_connexion(1, null, [player1[0], player2[0]]);
    }
}

const start_tournament = async (tournament_info) => {
    tournament_players_info = tournament_info;
    await tournament_algorithm()
}

const select_tournament_mode = () => {
        delete_app_element();
        create_element('div', 'app', null, 'tournament_wrapper').classList.add('tournament_wrapper');
        create_element('div', 'tournament_wrapper', null, 'inputs_wrapper').classList.add('inputs_wrapper');
        let tournament_player_owner1 = create_element('input', 'inputs_wrapper', null, 'tournament_username_input1');
        let tournament_player2 = create_element('input', 'inputs_wrapper', null, 'tournament_username_input2');
        let tournament_player3 = create_element('input', 'inputs_wrapper', null, 'tournament_username_input3');
        let tournament_player4 = create_element('input', 'inputs_wrapper', null, 'tournament_username_input4');
        if (get_tournament_username_from_cookie())
            tournament_player_owner1.value = get_tournament_username_from_cookie();
        create_button('tournament_wrapper', 'Launch tournament', () => start_tournament([
            [tournament_player_owner1.value, 0, 0], [tournament_player2.value, 0, 0], [tournament_player3.value, 0, 0], [tournament_player4.value, 0, 0]
        ]), 'submit_button');
}

export const show_tournament_score = () => {
    delete_app_element();
    create_element('div', 'app', null, 'score_table');
    let tmp_scores = tournament_players_info.slice();
    let sorted_scores = [];
    while (tmp_scores.length > 0) {
        let tmp = null;
        let index_to_remove = 0;
        let index = 0;
        for (const score of tmp_scores) {
            if (tmp === null || score[1] > tmp[1]) {
                tmp = score;
                index_to_remove = index;
            }
            index++;
        }
        sorted_scores.push(tmp);
        tmp_scores.splice(index_to_remove, 1);
    }
    let i = 0;
    for (const score of sorted_scores) {
        let player = create_element('div', 'score_table', null, i + 'player_score');
        player.classList.add("player_score");
        create_element('p', i + 'player_score', score[0], null);
        create_element('p', i + 'player_score', score[1] + '', null);
        i++;
    }
}

export const choose_mode = async () => {
    if (!get_username_from_cookie()) {
        await navigate_to("/account");
    } else {
        if (in_queue)
            create_button('app', 'UNREGISTER', async () => unregister_multi(), 'unregister').classList.add("play_button");
        else {
            create_button('app', 'LOCAL', () => select_solo_mode(), 'local_mode_button').classList.add("play_button");
            create_button('app', 'MULTI', () => establish_connexion(0), 'multy_mode_button').classList.add("play_button");
            create_button('app', 'TOURNAMENT', () => select_tournament_mode(), 'Tournament_mode_button').classList.add("play_button");
        }
    }
}
