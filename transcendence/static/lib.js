export const create_element = (type, position = null, text = null, id = null) => {
    const element= document.createElement(type);
    if (id)
        element.id = id;
    if (text)
        element.innerText = text;
    if (position && document.getElementById(position))
        document.getElementById(position).appendChild(element);
    return element;
}

export const create_button = (position, text, on_click_fn = null, id = null) => {
    const button = create_element('button', position, text, id);
    if (on_click_fn !== null)
        button.onclick = on_click_fn;
    return button;
}

export const create_image = (position, text, src = null, alt = null, id = null) => {
    const image = create_element('img', position, text, id);
    if (src !== null)
        image.src = src;
    if (alt !== null)
        image.alt = alt;
    return image;
}

export const sleep = (ms) => {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export const delete_app_element = () => {
    document.getElementById("app").innerHTML = "";
}