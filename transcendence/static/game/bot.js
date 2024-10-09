
const LEFT = 0;
const RIGHT = 1;
const BALL_SPEED = 0.0115;
let tab_pos = [[0.65 - 0.0085, 0.5 - 0.0085]];
let tab_pred = [];
let pos_bot = 0.5;

const get_dist_wall = (dir, pos) => {
    let dist_l_or_r;
    let dist_t_or_b;
    if (dir[0] > 0)
        dist_l_or_r = 1.268 - pos[0];
    else
        dist_l_or_r = pos[0] - 0.015;
    if (dir[1] > 0)
        dist_t_or_b = 0.983 - pos[1];
    else
        dist_t_or_b = pos[1];
    return [dist_l_or_r, dist_t_or_b];
}

const pred_next_bounce_from_bottom = (pred, tab_pred_u, side) => {
    pred = 0.983 - (pred - 0.983);
    let pos_bot_u = 0.5;
    if (side === RIGHT) {
        tab_pred_u.push([1.268, pred]);
        pos_bot_u = pred;
    } else
        tab_pred_u.push([0.015, pred]);
    return [tab_pred_u, pos_bot_u];
}

const pred_next_bounce_from_top= (pred, tab_pred_u, side) => {
    pred = - pred;
    let pos_bot_u = 0.5;
    if (side === RIGHT) {
        tab_pred_u.push([1.268, pred]);
        pos_bot_u = pred;
    } else
        tab_pred_u.push([0.015, pred]);
    return [tab_pred_u, pos_bot_u];
}

const add_collision_pred = (dir, actual_point) => {
    let dist_walls = get_dist_wall(dir, actual_point);
    let pos_bot_u = 0.5;
    let tab_pred_u = [];
    if (Math.abs(dist_walls[0] / dir[0]) < Math.abs(dist_walls[1] / dir[1])) {
        if (dir[0] > 0) {
            tab_pred_u.push([1.268, actual_point[1] + dir[1] * ((-actual_point[0] + 1.268) / dir[0])]);
            pos_bot_u = actual_point[1] + dir[1] * ((-actual_point[0] + 1.268) / dir[0]);
        } else {
            tab_pred_u.push([0.015, actual_point[1] + dir[1] * ((-actual_point[0] + 0.015) / dir[0])]);
        }
    }
    else {
        if (dir[1] > 0) {
            tab_pred_u.push([actual_point[0] + dir[0] * ((-actual_point[1] + 0.983) / dir[1]), 0.983]);
            if (dir[0] > 0)
                [tab_pred_u, pos_bot_u] = pred_next_bounce_from_bottom(actual_point[1] + dir[1] * ((-actual_point[0] + 1.268) / dir[0]), tab_pred_u, RIGHT);
            else
                [tab_pred_u, pos_bot_u] = pred_next_bounce_from_bottom(actual_point[1] + dir[1] * ((-actual_point[0] + 0.015) / dir[0]), tab_pred_u, LEFT);
        } else {
            tab_pred_u.push([actual_point[0] + dir[0] * ((-actual_point[1]) / dir[1]), 0]);
            if (dir[0] > 0)
                [tab_pred_u, pos_bot_u] = pred_next_bounce_from_top(actual_point[1] + dir[1] * ((-actual_point[0] + 1.268) / dir[0]), tab_pred_u, RIGHT);
            else
               [tab_pred_u, pos_bot_u] = pred_next_bounce_from_top(actual_point[1] + dir[1] * ((-actual_point[0] + 0.015) / dir[0]), tab_pred_u, LEFT);
        }
    }
    return [tab_pred_u, pos_bot_u];
}

const get_ball_dir = (pos_t_minus_1, actual_pos) => {
    return [actual_pos[0] - pos_t_minus_1[0], actual_pos[1] - pos_t_minus_1[1]];
}

const distance_between_two_point = (first, second) => {
    return Math.sqrt(Math.pow(second[0] - first[0], 2) + Math.pow(second[1] - first[1], 2));
}

const wall_bounce_between_last_pad_bounce_and_refresh = (previous_pos, actual_pos, length) => {
    let x_pos_wall_bounce
    if (previous_pos[1] > 0.5) {
        x_pos_wall_bounce = (0.983 - previous_pos[1])
            / (Math.tan(Math.asin((0.983 - previous_pos[1] + 0.983 - actual_pos[1]) / length)));
    } else {
        x_pos_wall_bounce = previous_pos[1]
            / (Math.tan(Math.asin((previous_pos[1] + actual_pos[1]) / length)));
    }
    if (Math.abs(previous_pos[0] - 0.015) < 0.001)
        x_pos_wall_bounce += 0.015;
    else
        x_pos_wall_bounce = 1.268 - x_pos_wall_bounce;
    return actual_pos[1] > 0.5 ? [x_pos_wall_bounce, 0.983] : [x_pos_wall_bounce, 0];
}

const update_arrays = (actual_pos, distance_run, tab_pred_u, tab_pos_u) => {
    if (Math.abs(Math.abs(distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], actual_pos)) - distance_run) > 0.0001) {
        if (tab_pred_u.length === 1) {
            tab_pos_u.push(tab_pred_u[0]);
            tab_pred_u.shift();

            const len_travel = distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                    + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], actual_pos)

            if (tab_pos_u.length >= 2 && len_travel - distance_run < -0.0001) {
                const tmp = wall_bounce_between_last_pad_bounce_and_refresh(tab_pos_u[tab_pos_u.length - 1], actual_pos,
                    distance_run - distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1]));

                if (!isNaN(tmp[0]) && Math.abs(len_travel - distance_run) > Math.abs(distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                    + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], tmp) + distance_between_two_point(tmp, actual_pos) - distance_run))
                    tab_pos_u[tab_pos_u.length - 1] = tmp;
            }
        } else if (tab_pred_u.length === 2) {
            tab_pos_u.push(tab_pred_u[0]);
            tab_pred_u.shift();

            const length_with_one_collision = Math.abs(distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], actual_pos));
            const length_with_two_collision = Math.abs(distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], tab_pred_u[0])
                + distance_between_two_point(tab_pred_u[0], actual_pos));

            if (Math.abs(length_with_one_collision - distance_run) > Math.abs(length_with_two_collision - distance_run)) {
                tab_pos_u.push(tab_pred_u[0]);
                tab_pred_u.shift();

                const len_travel = distance_between_two_point(tab_pos_u[tab_pos_u.length - 3], tab_pos_u[tab_pos_u.length - 2])
                        + distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                        + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], actual_pos);

                if (tab_pos_u.length >= 3 && len_travel - distance_run < -0.0001) {
                    const tmp = wall_bounce_between_last_pad_bounce_and_refresh(tab_pos_u[tab_pos_u.length - 1], actual_pos,
                        distance_run - distance_between_two_point(tab_pos_u[tab_pos_u.length - 3], tab_pos_u[tab_pos_u.length - 2])
                         - distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1]));

                    if (!isNaN(tmp[0]) && Math.abs(len_travel - distance_run) > Math.abs(distance_between_two_point(tab_pos_u[tab_pos_u.length - 3], tab_pos_u[tab_pos_u.length - 2])
                        + distance_between_two_point(tab_pos_u[tab_pos_u.length - 2], tab_pos_u[tab_pos_u.length - 1])
                        + distance_between_two_point(tab_pos_u[tab_pos_u.length - 1], tmp) + distance_between_two_point(tmp, actual_pos) - distance_run))
                        tab_pos_u[tab_pos_u.length - 1] = tmp;
                }
            }
        }
    }
    return [tab_pred_u, tab_pos_u];
}

export const determine_bot_pos = (ball, nb_frame) => {
    [tab_pred, tab_pos] = update_arrays(ball, BALL_SPEED * nb_frame, tab_pred, tab_pos);
    tab_pos.push(ball);
    tab_pred = [];
    [tab_pred, pos_bot] = add_collision_pred(get_ball_dir(tab_pos[tab_pos.length - 2], tab_pos[tab_pos.length - 1]), tab_pos[tab_pos.length - 1]);
    return pos_bot + Math.random() / 15 - 0.5 / 15;
}

export const reset_tab_pos = () => {
    tab_pos = [[0.65 - 0.0085, 0.5 - 0.0085]];
    pos_bot = 0.5;
    tab_pred = [];
}
