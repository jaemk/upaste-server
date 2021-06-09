/** edit.js
 * - Handles creating, editing, and viewing pastes.
 *
 */

VIEW_BASE_URL = "/";

document.addEventListener("DOMContentLoaded", function() {
    var save   = document.getElementById("save-paste");     // save-paste button/element
    var edit   = document.getElementById("edit-paste");     // edit-paste button/element

    var pasteType = document.getElementById("paste-type");          // ace-editor mode (syntax)
    var typeSelector = document.getElementById("type-selector");    // select ace-editor mode
    var encryptionKeyInput = document.getElementById("encryption-key");    // select encryption-key password
    var pasteId = document.getElementById("paste-id");              // existing paste-id
    var copyLink = document.getElementById("copy-link");                   // share button
    var copyCode = document.getElementById("copy-code");                   // share button
    var encryptionKeyRequired = !!document.getElementById("encryption-key-required");
    var decryptionKeyInput = document.getElementById("decryption-key");   // decryption pass
    var decryptPaste = document.getElementById("decrypt-paste");     // decrypt button
    var editorElem = document.getElementById("editor");

    if (encryptionKeyRequired) {
        edit.style.display = "none";
    }

    // initialize editor with theme
    var editor = ace.edit("editor");
    editor.setTheme("ace/theme/tomorrow_night_eighties");

    // update ace-mode when selected
    typeSelector.addEventListener('change', function() {
        var value = typeSelector.value;
        editor.session.setMode('ace/mode/'+value);
    });

    /** Decrypt content
     */
    var didDecrypt = false;
    function doDecrypt() {
        if (didDecrypt) { return; }
        didDecrypt = true;
        var _decKey = decryptionKeyInput.value;
        var _pasteKey = pasteId.innerText;

        var http = new XMLHttpRequest();
        var url  = "/json/"+_pasteKey;
        http.open("GET", url, true);
        http.setRequestHeader("x-upaste-encryption-key", _decKey);
        http.onreadystatechange = function() {
            if (http.readyState !== XMLHttpRequest.DONE) { return; }
            if (http.status != 200) {
                didDecrypt = false;
                alert("Error decrypting paste.");
                return;
            }
            var resp = JSON.parse(http.responseText);
            if (resp.paste) {
                var content = resp.paste.content;
                editor.setValue(content);
                editor.session.setMode('ace/mode/'+resp.paste.content_type);
                typeSelector.value = resp.paste.content_type;
                decryptionKeyInput.style.display = "none";
                decryptPaste.style.display = "none";
                edit.style.display = "";
                editorElem.style.top = "70";

                for (var i = 0, len = typeSelector.length; i < len; i++) {
                    if (typeSelector[i].value.trim() === resp.paste.content_type.trim()) {
                        typeSelector[i].selected = true;
                        break;
                    }
                }
            }
            else {
                didDecrypt = false;
                alert("Error decrypting paste.");
            }
        }
        http.send();
    }
    if (decryptionKeyInput) {
        decryptionKeyInput.addEventListener("keyup", function(ev) {
            if (ev.key === "Enter") {
                doDecrypt();
            }
        });
    }
    if (decryptPaste) {
        decryptPaste.addEventListener("click", function() {
            doDecrypt();
        });
    }

    /** Save content
     * - When the save button is present (which it should always be, might just be hidden),
     *   add a listener to post current content and redirect to a viewable link
     */
    var didSave = false;
    function doSave() {
        if (didSave) { return; }
        didSave = true;
        var content = editor.getValue();
        var contentType = typeSelector.value;
        if (!contentType) { contentType = "text" }
        var encryptionKey = encryptionKeyInput.value;
        var hasKey = !(encryptionKey === "" || encryptionKey === null || encryptionKey === undefined);

        var http = new XMLHttpRequest();
        var url  = "/new?type="+contentType;
        http.open("POST", url, true);
        http.setRequestHeader("Content-Type", "text/plain");
        if (hasKey) {
            http.setRequestHeader("x-upaste-encryption-key", encryptionKey);
        }
        http.onreadystatechange = function() {
            if (http.readyState !== XMLHttpRequest.DONE) { return; }
            if (http.status != 200) {
                didSave = false;
                alert("Error posting paste.");
                return;
            }
            var resp = JSON.parse(http.responseText);
            if (resp.key) {
                window.location.href = VIEW_BASE_URL+resp.key;
            }
            else {
                didSave = false;
                alert("Error posting paste.");
            }
        }
        http.send(content);
    }
    if (encryptionKeyInput) {
        encryptionKeyInput.addEventListener("keyup", function(ev) {
            if (ev.key === "Enter") {
                doSave();
            }
        });
    }
    if (save) {
        save.addEventListener("click", function(){
            doSave();
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
            edit.style.display = "none";
            save.style.display = "";
            var key = document.getElementById("paste-id");
            key.innerText = '';
            editor.setReadOnly(false);

            // show the type selector and encryption password fields
            typeSelector.style.display = "";

            for (var i = 0, len = typeSelector.length; i < len; i++) {
                if (typeSelector[i].value.trim() === pasteType.value.trim()) {
                    typeSelector[i].selected = true;
                    break;
                }
            }

            encryptionKeyInput.style.display = "";
            encryptionKeyInput.value = "";

            copyLink.style.cssText = "display: none;";
            copyCode.style.cssText = "display: none;";
        });
    }

    /** Copy links and codes
     */
    if (pasteId) {
        var copyLinkText = copyLink.innerText;
        var copyCodeText = copyCode.innerText;
        copyLink.addEventListener("click", function() {
            navigator.clipboard.writeText(window.location.host + VIEW_BASE_URL + pasteId.innerText.trim());
            copyLink.innerText = copyLinkText + " ✓";
            copyCode.innerText = copyCodeText;
        });
        copyCode.addEventListener("click", function() {
            navigator.clipboard.writeText(pasteId.innerText.trim());
            copyCode.innerText = copyCodeText + " ✓";
            copyLink.innerText = copyLinkText;
        });
    }

    // set ace mode if a content_type is specified
    if (pasteType && pasteType.value) {
        editor.session.setMode('ace/mode/'+pasteType.value);
    }

});
