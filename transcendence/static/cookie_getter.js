export function get_cookie(name) {
    if (document.cookie === null || document.cookie === '')
        return null;
    const found_cookie = document.cookie
        .split(';')
        .map((c) => c.trim())
        .find((c) => c.substring(0, name.length + 1) === (name + '='));
    if (found_cookie == null)
        return null;
    const cookie_value = found_cookie.substring(name.length + 1);
    return decodeURIComponent(cookie_value);
}

export const get_csrf_token = () => get_cookie('csrftoken');

export const get_username_from_cookie = () => get_cookie('username');

export const get_tournament_username_from_cookie = () => get_cookie('tournament_username');

export const get_pfp_uri_from_cookie = () => get_cookie('pfp_uri');
