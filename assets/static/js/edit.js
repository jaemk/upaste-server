/** edit.js
 * - Handles creating, editing, and viewing pastes.
 *
 */

VIEW_BASE_URL = "/p/";

document.addEventListener("DOMContentLoaded", function() {
    var save   = document.getElementById("save-paste");     // save-paste button/element
    var edit   = document.getElementById("edit-paste");     // edit-paste button/element

    var pasteType = document.getElementById("paste-type");          // ace-editor mode (syntax)
    var typeSelector = document.getElementById("type-selector");    // select ace-editor mode
    var pasteId = document.getElementById("paste-id");              // existing paste-id
    var share = document.getElementById("share");                   // share button

    // initialize editor with theme
    var editor = ace.edit("editor");
    editor.setTheme("ace/theme/tomorrow_night_eighties");

    // update ace-mode when selected
    typeSelector.addEventListener('change', function() {
        var value = typeSelector.value;
        editor.session.setMode('ace/mode/'+value);
    });

    /** Save content
     * - When the save button is present (which it should always be, might just be hidden),
     *   add a listener to post current content and redirect to a viewable link
     */
    if (save) {
        save.addEventListener("click", function(){
            var content = editor.getValue();
            var contentType = typeSelector.value;
            if (!contentType) { contentType = "text" }

            var http = new XMLHttpRequest();
            var url  = "/new?type="+contentType;
            http.open("POST", url, true);
            http.setRequestHeader("Content-Type", "text/plain");
            http.onreadystatechange = function() {
                if (http.readyState !== XMLHttpRequest.DONE) { return; }
                if (http.status != 200) {
                    alert("Error posting paste.");
                    return;
                }
                var resp = JSON.parse(http.responseText);
                if (resp.key) {
                    window.location.href = VIEW_BASE_URL+resp.key;
                }
                else {
                    alert("Error posting paste.");
                }
            }
            http.send(content);
        });
    }


    /** Edit existing content
     * - When the edit button is present:
     *   - make editor readonly
     *   - add listener to allow editing
     *   - add listener to hide previous paste-key, hide the edit button, and show save button
     */
    if (edit) {
        editor.setReadOnly(true);
        edit.addEventListener("click", function(){
            edit.style.cssText = "display: none;";
            save.style.cssText = "";
            var key = document.getElementById("paste-id");
            key.innerText = '';
            editor.setReadOnly(false);

            typeSelector.style.cssText = "";
            typeSelector.value = pasteType.value;
            share.style.cssText = "display: none;";
        });
    }


    if (pasteId && share) {
        share.addEventListener("click", function(){
            window.prompt("Copy to clipboard", window.location.host + VIEW_BASE_URL + pasteId.innerText);
        });
    }

    // set ace mode if a content_type is specified
    if (pasteType && pasteType.value) {
        editor.session.setMode('ace/mode/'+pasteType.value);
    }

});
