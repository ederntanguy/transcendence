import { set_player_direction, reset_player_direction } from "./wall.js"
import { BALL_EDGE, PAD_HEIGHT, PAD_WIDTH, RATIO } from "./game_engine.js";
import { create_element } from "../lib.js";

export const render_ball = (ball, render_target, element) => {
    if (!element)
        return
    element.style.left = (render_target.x + (ball.x / RATIO * render_target.width)) + 'px';
    element.style.top = (render_target.y + (ball.y * render_target.height)) + 'px';
    element.style.width = (BALL_EDGE * render_target.height) + 'px';
    element.style.height = (BALL_EDGE * render_target.height) + 'px';
}

export const render_wall = (wall, render_target, element) => {
    if (!element)
        return
    element.style.left = (render_target.x + (wall.x / RATIO * render_target.width)) + 'px';
    element.style.top = (render_target.y + (wall.y * render_target.height)) + 'px';
    element.style.width = (PAD_WIDTH * render_target.height) + 'px';
    element.style.height = (PAD_HEIGHT * render_target.height) + 'px';
}

export const create_ball = (game_element) => {
    let ball = document.createElement('img');
    ball.id = 'ball';
    ball.style.position = 'absolute';
    ball.src = 'static/game/assets/Ball.png';
    game_element.appendChild(ball);
}

const create_wall = (id) => {
    let wall = document.createElement('img');
    wall.id = id;
    wall.style.position = 'absolute';
    wall.src = 'static/game/assets/Wall.png';
    return wall;
}

export const create_walls = (game_element) => {
    window.removeEventListener('keydown', set_player_direction);
    window.addEventListener('keydown', set_player_direction, {capture:true});
    window.removeEventListener('keyup', reset_player_direction);
    window.addEventListener('keyup', reset_player_direction, {capture:true});
    game_element.appendChild(create_wall('left_wall'));
    game_element.appendChild(create_wall('right_wall'));
}

export const make_render_target = () => {
    let game_element = create_element('div', 'app', null, 'playground');
    const height = Math.round(window.innerHeight / 1.5);
    game_element.style.cssText = `position: absolute; left: ${window.innerWidth / 2 - Math.round(height * RATIO) / 2};
        top: ${window.innerHeight / 2 - height / 2}px; width: ${Math.round(height * RATIO)}px; height: ${height}px;`;
    return {
        x: 0,
        y: 0,
        height: height,
        width: Math.round(height * RATIO),
    };
}
