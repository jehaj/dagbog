let uploadButton = document.getElementById("save-button");
let titleElement = document.getElementById("input-title");
let textElement = document.getElementById("input-body-text");
uploadButton.addEventListener("click", async event => {
    let text = textElement.value;
    let title = titleElement.value;
    let res = await fetch("/new_entry", {
        method: "POST",
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            title,
            text
        }),
    });
});