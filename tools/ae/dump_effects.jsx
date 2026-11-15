// AEP Downgrader - tester kit: dump_effects.jsx
// Writes effects_<major>.<minor>_<win|mac>.json with every effect installed in this After Effects.
// Run via File > Scripts > Run Script File... ES3 / ExtendScript.

// Unattended runs (automation): set $.global.AEPD_OUT_DIR to a folder path
// before running this script. No dialogs are shown then; the final message is
// stored in $.global.AEPD_LAST_MESSAGE instead.
function aepdUnattended() {
    return typeof $.global.AEPD_OUT_DIR === "string" && $.global.AEPD_OUT_DIR !== "";
}
function aepdPickFolder(prompt) {
    if (aepdUnattended()) { return new Folder($.global.AEPD_OUT_DIR); }
    return Folder.selectDialog(prompt);
}
function aepdAlert(msg) {
    if (aepdUnattended()) { $.global.AEPD_LAST_MESSAGE = String(msg); return; }
    alert(msg);
}

function osTag() { return ($.os.indexOf("Windows") >= 0) ? "win" : "mac"; }

function fileAccessOn() {
    try {
        return app.preferences.getPrefAsLong("Main Pref Section", "Pref_SCRIPTING_FILE_NETWORK_SECURITY") == 1;
    } catch (e) { return true; }
}

function jsonStr(s) {
    var out = "\"";
    var i, c, code, h;
    s = String(s);
    for (i = 0; i < s.length; i++) {
        c = s.charAt(i);
        code = s.charCodeAt(i);
        if (c === "\"") { out += "\\\""; }
        else if (c === "\\") { out += "\\\\"; }
        else if (c === "\n") { out += "\\n"; }
        else if (c === "\r") { out += "\\r"; }
        else if (c === "\t") { out += "\\t"; }
        else if (code < 32 || code > 126) {
            h = code.toString(16);
            while (h.length < 4) { h = "0" + h; }
            out += "\\u" + h;
        } else { out += c; }
    }
    return out + "\"";
}
function toJSON(v, ind) {
    var pad = "", i, parts, k, nl, nextInd;
    if (ind === undefined) { ind = ""; }
    nextInd = ind + "  ";
    if (v === null || v === undefined) { return "null"; }
    if (typeof v === "string") { return jsonStr(v); }
    if (typeof v === "number") { return isFinite(v) ? String(v) : "null"; }
    if (typeof v === "boolean") { return v ? "true" : "false"; }
    if (v instanceof Array) {
        if (v.length === 0) { return "[]"; }
        parts = [];
        for (i = 0; i < v.length; i++) { parts.push(nextInd + toJSON(v[i], nextInd)); }
        return "[\n" + parts.join(",\n") + "\n" + ind + "]";
    }
    parts = [];
    for (k in v) {
        if (v.hasOwnProperty(k)) { parts.push(nextInd + jsonStr(k) + ": " + toJSON(v[k], nextInd)); }
    }
    if (parts.length === 0) { return "{}"; }
    return "{\n" + parts.join(",\n") + "\n" + ind + "}";
}

function main() {
    var errors = [];
    if (!fileAccessOn()) {
        aepdAlert("AEPD dump effects: scripts are not allowed to write files.\n\n" +
            "Turn it on: Edit > Preferences > Scripting & Expressions (Mac: After Effects > Settings > Scripting & Expressions), " +
            "tick \"Allow Scripts to Write Files and Access Network\", then run again.");
        return;
    }
    var folder = aepdPickFolder("AEPD dump effects: choose the output folder");
    if (!folder) { return; }

    var s = String(app.version);
    var parts = s.split(".");
    var maj = parseInt(parts[0], 10), min = parseInt(parts[1], 10);
    if (isNaN(maj)) { maj = 0; }
    if (isNaN(min)) { min = 0; }
    var tag = maj + "." + min + "_" + osTag();

    var info = { appVersion: s, buildName: null, os: $.os, effectCount: 0, effects: [] };
    try { info.buildName = String(app.buildName); } catch (e1) { errors.push("buildName: " + e1); }

    var list = [];
    try {
        var effs = app.effects;
        var i, e;
        for (i = 0; i < effs.length; i++) {
            try {
                e = effs[i];
                list.push({
                    displayName: String(e.displayName),
                    matchName: String(e.matchName),
                    category: String(e.category)
                });
            } catch (e2) { errors.push("effect " + i + ": " + e2); }
        }
    } catch (e3) { errors.push("app.effects: " + e3); }
    list.sort(function (a, b) { return (a.matchName < b.matchName) ? -1 : ((a.matchName > b.matchName) ? 1 : 0); });
    info.effects = list;
    info.effectCount = list.length;
    if (errors.length) { info.errors = errors; }

    var path = folder.fsName + "/effects_" + tag + ".json";
    var ok = false;
    try {
        var f = new File(path);
        f.encoding = "UTF-8";
        if (f.open("w")) { f.write(toJSON(info, "") + "\n"); f.close(); ok = true; }
    } catch (e4) { errors.push("write: " + e4); }

    aepdAlert(ok ? ("AEPD dump effects\n\n" + list.length + " effects written to:\n" + path +
        (errors.length ? ("\n\nProblems: " + errors.length + " (listed inside the file)") : ""))
        : ("AEPD dump effects: could not write " + path + "\n" + errors.join("\n")));
}

main();
