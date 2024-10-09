import { PAD_HEIGHT, PAD_WIDTH, RATIO } from "./game_engine.js";

export let l_up = 0;
export let r_up = 0;
export let l_down = 0;
export let r_down = 0;

export const make_wall_structs = () => {
    return [
        {x: 0.0, y: 0.5 - PAD_HEIGHT / 2.0},
        {x: RATIO - PAD_WIDTH, y: 0.5 - PAD_HEIGHT / 2.0},
    ];
}

export const set_player_direction = (event) => {
    event.preventDefault();
    const key = event.key;
    if (key === 'w')
        l_up = 1;
    else if (key === 's')
        l_down = 1;
    else if (key === 'ArrowUp')
        r_up = 1;
    else if (key === 'ArrowDown')
        r_down = 1;
}

export const reset_player_direction = (event) => {
    event.preventDefault();
    const key = event.key;
    if (key === 'w')
        l_up = 0;
    else if (key === 's')
        l_down = 0;
    else if (key === 'ArrowUp')
        r_up = 0;
    else if (key === 'ArrowDown')
        r_down = 0;
}
