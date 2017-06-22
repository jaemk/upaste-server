/** edit.js
 * - Handles creating, editing, and viewing pastes.
 *
 */

/// Getting and setting cursor position inside a contenteditable element:
/// http://stackoverflow.com/a/41034697
function createRange(node, chars, range) {
    if (!range) {
        range = document.createRange()
        range.selectNode(node);
        range.setStart(node, 0);
    }
    if (chars.count === 0) {
        range.setEnd(node, chars.count);
    } else if (node && chars.count >0) {
        if (node.nodeType === Node.TEXT_NODE) {
            if (node.textContent.length < chars.count) {
                chars.count -= node.textContent.length;
            } else {
                range.setEnd(node, chars.count);
                chars.count = 0;
            }
        } else {
           for (var lp = 0; lp < node.childNodes.length; lp++) {
                range = createRange(node.childNodes[lp], chars, range);

                if (chars.count === 0) {
                    break;
                }
            }
        }
    }
    return range;
};
function setCurrentCursorPosition(editor, chars) {
    if (chars >= 0) {
        var selection = window.getSelection();

        range = createRange(editor, { count: chars });

        if (range) {
            range.collapse(false);
            selection.removeAllRanges();
            selection.addRange(range);
        }
    }
};
function isChildOf(node, parentId) {
    while (node !== null) {
        if (node.id === parentId) {
            return true;
        }
        node = node.parentNode;
    }

    return false;
};
function getCurrentCursorPosition(parentId) {
    var selection = window.getSelection(),
        charCount = -1,
        node;

    if (selection.focusNode) {
        if (isChildOf(selection.focusNode, parentId)) {
            node = selection.focusNode;
            charCount = selection.focusOffset;

            while (node) {
                if (node.id === parentId) {
                    break;
                }

                if (node.previousSibling) {
                    node = node.previousSibling;
                    charCount += node.textContent.length;
                } else {
                     node = node.parentNode;
                     if (node === null) {
                         break
                     }
                }
            }
        }
    }
    return charCount;
};



/**
 * toHljsClass
 * - handle exceptions where list value differs from class name
 *   that hljs expects
 *
 * @param {string} s - paste_type
 * @returns {string} - hljs class
 */
function toHljsClass(s) {
    if (s === "html/xml") { return "xml"; }
    if (s === "auto") { return null; }
    return s;
}


document.addEventListener("DOMContentLoaded", function() {
    var editor = document.getElementById("editor");         // editor container
    var save   = document.getElementById("save-paste");     // save-paste button/element
    var edit   = document.getElementById("edit-paste");     // edit-paste button/element
    var gutter = document.getElementById("gutter");         // line-num gutter
    var pasteText = document.getElementById("paste-text");  // hidden input holding paste content on-load
    var pasteType = document.getElementById("paste-type");
    var typeSelector = document.getElementById("type-selector");
    var pasteId = document.getElementById("paste-id");
    var share = document.getElementById("share");

    // set initial gutter and editor content
    gutter.innerText = '1 >';
    editor.innerText = "\n";

    /**
     * handleInput
     * - Handles changes to editor content from either an "input" `event` on the
     *   editor or an explicit call providing an `_element` and updates the gutter line nums.
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


    /** Handle clipboard pasting
     *
     */
    editor.addEventListener("paste", function(e){
        if (!editor.getAttribute('contenteditable')) { return; }
        e.stopPropagation();
        e.preventDefault();

        var clipboardData = e.clipboardData || window.clipboardData;
        var pastedData = clipboardData.getData('Text');

        var pos = getCurrentCursorPosition('editor');
        var head = editor.innerText.slice(0, pos);
        var tail = editor.innerText.slice(pos);
        editor.innerText = head + pastedData.replace(/ /g, '\u00a0') + tail;
        setCurrentCursorPosition(editor, head.length + pastedData.length);
        // update gutter
        handleInput(null, editor);
    });


    /** Handle tabs
     *
     */
    editor.addEventListener('keydown', function(e) {
        if (e.keyCode !== 9 || !editor.getAttribute('contenteditable')) { return; }
        e.stopPropagation();
        e.preventDefault();
        var tabWidth = parseInt(document.getElementById('tab-width'));
        if (!tabWidth || isNaN(tabWidth)) { tabWidth = 4; }
        var pos = getCurrentCursorPosition('editor');
        var head = editor.innerText.slice(0, pos);
        var tail = editor.innerText.slice(pos);
        var tab = '\u00a0'.repeat(tabWidth);
        editor.innerText = head + tab + tail;
        setCurrentCursorPosition(editor, head.length + tab.length);
        handleInput(null, editor);
    });


    /** Handle trailing <br>'s that get randomly inserted.
     *  - These appear when backspacing over multiple lines.
     *    If a tab is inserted when the cursor is on the <br>,
     *    the tab-spaces are inserted at a random location.
     *  - replacing trailing breaks with new-lines and make sure
     *    the cursor stays at the end of the text.
     */
    editor.addEventListener('input', function(e){
        var brs = editor.querySelectorAll('br');
        if (brs.length > 0) {
            for (var i = 0; i < brs.length; i++) {
                editor.removeChild(brs[i]);
                editor.innerText += '\n';
                setCurrentCursorPosition(editor, editor.innerText.length);
            }
            handleInput(null, editor);
        }
    });


    /** Save content
     * - When the save button is present (which it should always be, might just be hidden),
     *   add a click listener to post current content and redirect to a viewable link
     */
    if (save) {
        save.addEventListener("click", function(){
            var editor = document.getElementById("editor");
            var content = editor.innerText.replace(/\u00a0/g, " ");
            var contentType = typeSelector.value;
            if (!contentType) { contentType = "text" }

            var http = new XMLHttpRequest();
            var url  = "/new?type="+contentType;
            http.open("POST", url, true);
            http.setRequestHeader("Content-Type", "text/plain");
            http.onreadystatechange = function() {
                if (http.readyState !== XMLHttpRequest.DONE) { return; }
                if (http.status >= 500) {
                    alert("Error posting paste.");
                    return;
                }
                var resp = JSON.parse(http.responseText);
                if (resp.key) {
                    window.location.href = "/"+resp.key;
                }
                else {
                    alert("Error posting paste.");
                }
            }
            http.send(content);
        });
    }


    /** Edit existing content
     * - When the edit button is present, add a click listener to change up the header area.
     *   - hide the edit button
     *   - show the save button
     *   - change the paste-id to a helpful message
     *   - make editor area editable
     *   - set the editor text to itself so it reformats from groups
     *     of elements to only text
     */
    if (edit) {
        edit.addEventListener("click", function(){
            edit.style.cssText = "display: none;";
            save.style.cssText = "";
            var key = document.getElementById("paste-id");
            key.innerText = '';
            editor.setAttribute('contenteditable', true);
            editor.innerText = editor.innerText;

            typeSelector.style.cssText = "";
            typeSelector.value = pasteType.value;
            share.style.cssText = "display: none;";
        });
    }


    /** Initial load paste content
     * - When viewing a paste, the raw paste-content will be in a hidden div 'paste-text'.
     *   Copy this text into the editor and pass the editor to `handleInput` to update the gutter.`
     */
    if (pasteText) {
        var text = pasteText.value;
        editor.innerText = text;
        handleInput(null, editor);
    }


    if (pasteId && share) {
        share.addEventListener("click", function(){
            window.prompt("Copy to clipboard", 'https://' + window.location.host + '/' + pasteId.innerText);
        });
    }

    if (pasteType) {
        var hljsClass = toHljsClass(pasteType.value);
        if (hljsClass) {
            editor.classList.add(hljsClass);
        }
        hljs.highlightBlock(editor);
    }
});
