/** edit.js
 * - Handles creating, editing, and viewing pastes.
 *
 */
document.addEventListener("DOMContentLoaded", function() {
    var editor = document.getElementById("editor");         // editor container
    var save   = document.getElementById("save-paste");     // save-paste button/element
    var edit   = document.getElementById("edit-paste");     // edit-paste button/element
    var gutter = document.getElementById("gutter");         // line-num gutter
    var pasteText = document.getElementById("paste-text");  // hidden input holding paste content on-load

    // set initial gutter and editor content
    gutter.innerText = '1 >';
    editor.innerText = "\n";

    /**
     * handleInput
     * - Handles changes to editor content from either an "input" `event` on the
     *   editor or an explicit call providing an `_element`
     *
     * @param event     - input event
     * @param _element  - optional target element. when specified, will use over event.target
     * @returns {undefined}
     */
    function handleInput(event, _element) {
        var element;
        if (_element) {
            element = _element;
        } else {
            event.preventDefault();
            element = event.target;
        }
        var content = element.innerText;
        var lines = content.split("\n");

        // hitting enter always inserts a double newline.
        // if the last line entry is blank, number one less line in the gutter.
        var numLines = lines.length;
        if (lines[numLines-1] === '') { numLines -= 1; }

        if (numLines === 0) { element.innerText = "\n"; }

        var gutterLines = [];
        for (var i=0; i<numLines; i++) {
            gutterLines.push((i+1)+ ' >');
        }
        if (gutterLines.length === 0) { gutterLines.push('1 >'); }
        gutter.innerText = gutterLines.join("\n");
    }
    var editor = document.getElementById('editor');
    editor.addEventListener("input", handleInput, false);

    /** Save
     * - When the save button is present (which it should always be, might just be hidden),
     *   add a click listener to post current content and redirect to a viewable link
     */
    if (save) {
        save.addEventListener("click", function(){
            var editor = document.getElementById("editor");
            var content = editor.innerText;

            console.log(content.split("\n"));
            var http = new XMLHttpRequest();
            var url  = "/new";
            http.open("POST", url, true);
            http.setRequestHeader("Content-Type", "text/plain");
            http.onreadystatechange = function() {
                var resp = JSON.parse(http.responseText);
                if (resp.key) { window.location.href = "/"+resp.key; }
                else {
                    alert("Error posting paste.");
                }
            }
            http.send(content);
        });
    }

    /** Edit
     * - When the edit button is present, add a click listener to change up the header area.
     *   - hide the edit button
     *   - show the save button
     *   - change the paste-id to a helpful message
     *   - make editor area editable
     */
    if (edit) {
        edit.addEventListener("click", function(){
            edit.style.cssText = "display: none;";
            save.style.cssText = "";
            var key = document.getElementById("paste-id");
            key.innerText = "Hit 'Save' when you're finished!";
            editor.setAttribute('contenteditable', true);
        });
    }

    /** Load paste content
     * - When viewing a paste, the raw paste-content will be in a hidden div 'paste-text'.
     *   Copy this text into the editor and pass the editor to `handleInput` to update the gutter.`
     */
    if (pasteText) {
        var text = pasteText.value;
        editor.innerText = text;
        handleInput(null, editor);
    }
});
