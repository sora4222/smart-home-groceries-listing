/**
 * Sends the typed server address to the app's `save_server_url` command.
 * On success the app restarts on the server; on a refusal the app's plain
 * sentence is shown under the box.
 */
const form = document.getElementById("setup");
const input = document.getElementById("server-url");
const problem = document.getElementById("problem");
const button = form.querySelector("button");

/** Shows `text` under the box, or hides the message when `text` is empty. */
function showProblem(text) {
	problem.textContent = text;
	problem.hidden = !text;
}

form.addEventListener("submit", async (event) => {
	event.preventDefault();
	showProblem("");
	button.disabled = true;
	button.textContent = "Saving…";
	try {
		await window.__TAURI_INTERNALS__.invoke("save_server_url", {
			url: input.value,
		});
	} catch (err) {
		console.error("[setup] address refused", err);
		showProblem(typeof err === "string" ? err : "The address could not be saved.");
		button.disabled = false;
		button.textContent = "Save and open";
	}
});
