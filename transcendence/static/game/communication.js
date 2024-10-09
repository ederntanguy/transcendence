import { Encoder } from "../encode_cbor.js";
import { Decoder } from "../decode_cbor.js";
import { get_username_from_cookie } from "../cookie_getter.js";
import { update_value_manager } from "./game_loop.js";
import { init_game, register_in_queue, show_tournament_score } from "./game_engine.js";
import { create_element } from "../lib.js";

const MODE_MULTY = 0;

let socket = null;

export const send_information = (msg) => {
    const encoder = new Encoder;
    if (socket !== null)
        socket.send(encoder.encode(msg));
}

export const close_connexion = () => {
    if (socket !== null) {
        socket.removeEventListener('close', close_listener);
        socket.close();
    }
    socket = null;
    return -2;
}

export const unexpected_quit = () => {
    if (socket !== null) {
        socket.removeEventListener('close', close_listener);
        socket.close();
    }
    socket = null;
}

const close_listener = () => {
        socket = null;
}

export const establish_connexion = (game_mode, is_PvE = null, tournament_players_username = null) => {
    if (socket !== null)
        return;
    socket = new WebSocket("wss://" + window.location.hostname + ":8081");
    if (game_mode === MODE_MULTY)
        register_in_queue();
    socket.binaryType = "arraybuffer";
    let decoder = new Decoder;

    socket.addEventListener("open", async (_) => {
        send_information([3, get_username_from_cookie(), game_mode]);
    });

    socket.addEventListener("close", close_listener);

    socket.addEventListener("message", async (event) => {
        socket.addEventListener("message", async (event) => {
            await update_value_manager(decoder.decode(new Uint8Array(event.data)));
        });
        await init_game(game_mode, decoder.decode(new Uint8Array(event.data)), is_PvE, tournament_players_username);
    }, {once: true});
}