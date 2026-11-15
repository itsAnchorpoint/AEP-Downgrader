// AEP Downgrader - tester kit: verify_open.jsx
// Opens every .aep / .aet in a folder (not recursive) and applies every .ffx preset,
// then writes verify_report_<major>.<minor>_<win|mac>.json into the same folder.
// Run via File > Scripts > Run Script File... ES3 / ExtendScript.
//
// About unresolved ("missing") effects: After Effects offers no reliable scripting flag for an
// effect that failed to load, so this script does NOT claim to detect them. Instead it lists, for
// each project, the distinct effect matchNames used, and (as a hint only) those that are not in
// this AE's app.effects list. Compare with the effects_*.json from dump_effects.jsx.

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
function errText(e) {
    try { return (e && e.toString) ? e.toString() : String(e); } catch (x) { return "unknown error"; }
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
    var i, parts, k, nextInd;
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
function lowerExt(name) {
    var i = name.lastIndexOf(".");
    return (i < 0) ? "" : name.substring(i).toLowerCase();
}
function sortStrings(a) { a.sort(function (x, y) { return (x < y) ? -1 : ((x > y) ? 1 : 0); }); }

// Inspect the currently open project.
function inspectProject() {
    var r = { items: 0, comps: 0, totalLayers: 0, totalEffects: 0, compLayers: [], effectMatchNames: [],
        effectMatchNamesNotInEffectsList: [] };
    var known = {};
    var seen = {};
    var i, j, k, it, lyr, parade, eff, mn, effs;
    try {
        effs = app.effects;
        for (i = 0; i < effs.length; i++) { known[String(effs[i].matchName)] = true; }
    } catch (e0) { known = null; }
    r.items = app.project.numItems;
    for (i = 1; i <= app.project.numItems; i++) {
        try {
            it = app.project.item(i);
            if (it instanceof CompItem) {
                r.comps++;
                r.compLayers.push({ name: it.name, layers: it.numLayers });
                r.totalLayers += it.numLayers;
                for (j = 1; j <= it.numLayers; j++) {
                    try {
                        lyr = it.layer(j);
                        parade = lyr.property("ADBE Effect Parade");
                        if (parade) {
                            r.totalEffects += parade.numProperties;
                            for (k = 1; k <= parade.numProperties; k++) {
                                try {
                                    eff = parade.property(k);
                                    mn = String(eff.matchName);
                                    seen[mn] = true;
                                } catch (e1) {}
                            }
                        }
                    } catch (e2) {}
                }
            }
        } catch (e3) {}
    }
    for (mn in seen) {
        if (seen.hasOwnProperty(mn)) {
            r.effectMatchNames.push(mn);
            if (known && !known[mn]) { r.effectMatchNamesNotInEffectsList.push(mn); }
        }
    }
    sortStrings(r.effectMatchNames);
    sortStrings(r.effectMatchNamesNotInEffectsList);
    if (!known) { delete r.effectMatchNamesNotInEffectsList; }
    return r;
}

// Count properties that have keyframes or an enabled expression, recursively.
function countAnimated(group, depth) {
    var n = 0, i, p;
    if (depth > 12) { return 0; }
    for (i = 1; i <= group.numProperties; i++) {
        try {
            p = group.property(i);
            if (p.propertyType === PropertyType.PROPERTY) {
                if (p.numKeys > 0) { n++; }
                else if (p.expressionEnabled) { n++; }
            } else {
                n += countAnimated(p, depth + 1);
            }
        } catch (e) {}
    }
    return n;
}

function main() {
    if (!fileAccessOn()) {
        aepdAlert("AEPD verify: scripts are not allowed to write files.\n\n" +
            "Turn it on: Edit > Preferences > Scripting & Expressions (Mac: After Effects > Settings > Scripting & Expressions), " +
            "tick \"Allow Scripts to Write Files and Access Network\", then run again.");
        return;
    }
    if (app.project && (app.project.numItems > 0 || app.project.file)) {
        aepdAlert("AEPD verify: a project is open.\n\nPlease close it (File > Close Project, do not save) and run the script again.");
        return;
    }
    var folder = aepdPickFolder("AEPD verify: choose the folder with the .aep / .aet / .ffx files");
    if (!folder) { return; }

    var s = String(app.version);
    var parts = s.split(".");
    var maj = parseInt(parts[0], 10), min = parseInt(parts[1], 10);
    if (isNaN(maj)) { maj = 0; }
    if (isNaN(min)) { min = 0; }
    var tag = maj + "." + min + "_" + osTag();

    var report = { appVersion: s, buildName: null, os: $.os, folder: folder.fsName, projects: [], presets: [], errors: [] };
    try { report.buildName = String(app.buildName); } catch (e1) {}

    var all = folder.getFiles();
    var aeps = [], ffxs = [];
    var i, f, nm, ext;
    for (i = 0; i < all.length; i++) {
        f = all[i];
        if (!(f instanceof File)) { continue; }
        nm = f.name;
        ext = lowerExt(nm);
        if (ext === ".aep" || ext === ".aet") { aeps.push(f); }
        else if (ext === ".ffx") { ffxs.push(f); }
    }
    aeps.sort(function (a, b) { return (a.name < b.name) ? -1 : ((a.name > b.name) ? 1 : 0); });
    ffxs.sort(function (a, b) { return (a.name < b.name) ? -1 : ((a.name > b.name) ? 1 : 0); });

    var opened = 0, failed = 0;
    for (i = 0; i < aeps.length; i++) {
        var rec = { file: aeps[i].name, opened: false, error: null };
        app.beginSuppressDialogs();
        try {
            app.open(aeps[i]);
            rec.opened = true;
            try {
                var info = inspectProject();
                var k;
                for (k in info) { if (info.hasOwnProperty(k)) { rec[k] = info[k]; } }
            } catch (e2) { rec.inspectError = errText(e2); }
        } catch (e3) {
            rec.error = errText(e3);
        }
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e4) { rec.closeError = errText(e4); }
        try { app.endSuppressDialogs(false); } catch (e5) {}
        if (rec.opened) { opened++; } else { failed++; }
        report.projects.push(rec);
    }

    // Presets: need a project to hold the temp comp.
    if (ffxs.length > 0) {
        app.beginUndoGroup("AEPD verify presets");
        try {
            var comp = app.project.items.addComp("AEPD_ffx_temp", 1920, 1080, 1, 5, 25);
            for (i = 0; i < ffxs.length; i++) {
                var prec = { file: ffxs[i].name, applied: false, error: null, effects: 0, animatedProperties: 0, effectMatchNames: [] };
                try {
                    var lyr = comp.layers.addSolid([0.5, 0.5, 0.5], "AEPD_ffx_solid", 1920, 1080, 1, 5);
                    try {
                        lyr.applyPreset(ffxs[i]);
                        prec.applied = true;
                    } catch (e6) { prec.error = errText(e6); }
                    try {
                        var par = lyr.property("ADBE Effect Parade");
                        prec.effects = par.numProperties;
                        var q;
                        for (q = 1; q <= par.numProperties; q++) { prec.effectMatchNames.push(String(par.property(q).matchName)); }
                    } catch (e7) { prec.effectsError = errText(e7); }
                    try { prec.animatedProperties = countAnimated(lyr, 0); } catch (e8) { prec.animatedError = errText(e8); }
                    try { lyr.remove(); } catch (e9) {}
                } catch (e10) { prec.error = errText(e10); }
                if (prec.applied) { opened++; } else { failed++; }
                report.presets.push(prec);
            }
        } catch (e11) { report.errors.push("preset test: " + errText(e11)); }
        try { app.endUndoGroup(); } catch (e12) {}
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e13) {}
    }

    report.summary = { projectsOpened: 0, projectsFailed: 0, presetsApplied: 0, presetsFailed: 0 };
    for (i = 0; i < report.projects.length; i++) {
        if (report.projects[i].opened) { report.summary.projectsOpened++; } else { report.summary.projectsFailed++; }
    }
    for (i = 0; i < report.presets.length; i++) {
        if (report.presets[i].applied) { report.summary.presetsApplied++; } else { report.summary.presetsFailed++; }
    }

    var path = folder.fsName + "/verify_report_" + tag + ".json";
    var wrote = false;
    try {
        var out = new File(path);
        out.encoding = "UTF-8";
        if (out.open("w")) { out.write(toJSON(report, "") + "\n"); out.close(); wrote = true; }
    } catch (e14) {}

    aepdAlert("AEPD verify\n\n" +
        "Projects/templates: " + report.summary.projectsOpened + " opened, " + report.summary.projectsFailed + " failed\n" +
        "Presets: " + report.summary.presetsApplied + " applied, " + report.summary.presetsFailed + " failed\n\n" +
        (wrote ? ("Report: " + path) : ("Could not write the report to " + path)));
}

main();
