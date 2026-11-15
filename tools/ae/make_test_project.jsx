// AEP Downgrader - tester kit: make_test_project.jsx
// Builds a deterministic test project that exercises as much of the .aep format as
// scripting allows, then saves it. Run via File > Scripts > Run Script File...
// Every feature is wrapped in its own try/catch; a failure is logged and the run continues.
// ES3 / ExtendScript: no let/const, no JSON, no arrow functions.

// ---------------------------------------------------------------------------
// Settings (edit here)
// ---------------------------------------------------------------------------
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

var INCLUDE_ALL_EFFECTS = true;   // build the "FX all" comp (applies every ADBE*/CC* effect)
var EFFECTS_PER_SOLID = 25;

// Effects that start long analysis or open dialogs when applied. Exact matchNames.
// Add more entries here (strings) if your AE hangs or shows a dialog on some effect.
var SKIP_EFFECT_MATCHNAMES = [
    "ADBE SubspaceStabilizer",
    "ADBE 3D Tracker",
    "ADBE Camera 3D Tracker",
    "ADBE Roto Brush & Refine Edge"
];
// Second explicit skip list: matchNames that opened a dialog or took very long in REAL runs.
// It is filled from the "SLOW  <matchName> <ms> ms (possible dialog)" lines of make_test_project_log_*.txt.
var SKIP_EFFECT_MATCHNAMES_DIALOG = [
    "ADBE Basic Text2",   // opens a text entry dialog (AE 24.5: 6.5 s)
    "ADBE Path Text",     // opens a text entry dialog (AE 24.5)
    "ADBE Numbers2"       // opens a font dialog
];
var SLOW_EFFECT_MS = 1500;
// Any effect whose matchName or displayName contains one of these (case-insensitive) is skipped too.
var SKIP_EFFECT_SUBSTRINGS = [
    "stabiliz", "tracker", "3d camera", "roto", "content-aware", "content aware",
    "finesse", "mocha", "refine edge", "refine soft", "remove grain", "scene edit",
    "lut", "lookup", "ocio", "cineon"
];

// ---------------------------------------------------------------------------
// Infrastructure
// ---------------------------------------------------------------------------
var LOG = [];
var FAILS = 0;
var SKIPS = 0;
var BACKWARD = [];

function logOk(msg) { LOG.push("OK    " + msg); }
function logInfo(msg) { LOG.push("INFO  " + msg); }
function logSkip(msg) { SKIPS++; LOG.push("SKIP  " + msg); }
function logFail(name, e) {
    FAILS++;
    var t;
    try { t = (e && e.toString) ? e.toString() : String(e); } catch (x) { t = "?"; }
    var ln = "";
    try { if (e && e.line) { ln = " (line " + e.line + ")"; } } catch (x2) {}
    LOG.push("FAIL  " + name + " :: " + t + ln);
}
// Run fn inside its own try/catch. Returns fn's result, or undefined on failure.
function step(name, fn) {
    try {
        var r = fn();
        logOk(name);
        return r;
    } catch (e) {
        logFail(name, e);
        return undefined;
    }
}
// Same but silent on success (for tiny sub-steps).
function tryq(name, fn) {
    try { return fn(); } catch (e) { logFail(name, e); return undefined; }
}

function osTag() { return ($.os.indexOf("Windows") >= 0) ? "win" : "mac"; }
function verParts() {
    var s = String(app.version);
    var p = s.split(".");
    var maj = parseInt(p[0], 10);
    var min = parseInt(p[1], 10);
    if (isNaN(maj)) { maj = 0; }
    if (isNaN(min)) { min = 0; }
    return { major: maj, minor: min, text: s };
}
function fileAccessOn() {
    try {
        return app.preferences.getPrefAsLong("Main Pref Section", "Pref_SCRIPTING_FILE_NETWORK_SECURITY") == 1;
    } catch (e) {
        return true; // cannot tell; writing will fail loudly later if it is really off
    }
}
function writeText(path, text) {
    var f = new File(path);
    f.encoding = "UTF-8";
    if (!f.open("w")) { return false; }
    f.write(text);
    f.close();
    return true;
}
function has(obj, key) { try { return (obj[key] !== undefined); } catch (e) { return false; } }
function arrHas(arr, x) { var i; for (i = 0; i < arr.length; i++) { if (arr[i] === x) { return true; } } return false; }

// ---------------------------------------------------------------------------
// Small helpers for building
// ---------------------------------------------------------------------------
var W = 1920, H = 1080, FPS = 25, DUR = 5;
var ROOT = {};          // folders
var COMPS = {};         // named comps
var COL = {
    red: [0.9, 0.1, 0.1], green: [0.1, 0.8, 0.2], blue: [0.1, 0.3, 0.9],
    yellow: [0.95, 0.85, 0.1], grey: [0.5, 0.5, 0.5], white: [1, 1, 1], black: [0, 0, 0]
};

function mkFolder(name, parent) {
    var f = app.project.items.addFolder(name);
    if (parent) { f.parentFolder = parent; }
    return f;
}
function mkComp(name, folder, dur, w, h) {
    var c = app.project.items.addComp(name, w || W, h || H, 1, dur || DUR, FPS);
    if (folder) { c.parentFolder = folder; }
    return c;
}
function solid(comp, name, color, w, h) {
    return comp.layers.addSolid(color, name, w || W, h || H, 1, comp.duration);
}
function pr(layer, matchName) { return layer.property(matchName); }
function xf(layer, matchName) { return layer.property("ADBE Transform Group").property(matchName); }
function xfAny(layer, names) {
    var i, p;
    for (i = 0; i < names.length; i++) {
        p = null;
        try { p = layer.property("ADBE Transform Group").property(names[i]); } catch (e) { p = null; }
        if (p) { return p; }
    }
    throw new Error("transform property not found: " + names.join("|"));
}
function setIf(group, names, value, label) {
    // Try each candidate matchName; set the first that exists. Returns true if set.
    var i, p;
    for (i = 0; i < names.length; i++) {
        p = null;
        try { p = group.property(names[i]); } catch (e) { p = null; }
        if (p) {
            try { p.setValue(value); return true; } catch (e2) { logFail("set " + (label || names[i]), e2); return false; }
        }
    }
    logSkip("property not found: " + (label || names.join("|")));
    return false;
}
function setText(layer, str) {
    var p = layer.property("ADBE Text Properties").property("ADBE Text Document");
    var td = p.value;
    td.text = str;
    p.setValue(td);
}
function mkShape(verts, inT, outT, closed) {
    var s = new Shape();
    s.vertices = verts;
    s.inTangents = inT;
    s.outTangents = outT;
    s.closed = closed;
    return s;
}
function newMarker(comment) {
    var m = new MarkerValue(comment);
    return m;
}
function matte(comp, target, matteLayer, type, label) {
    // AE 23+ API when present, legacy trackMatteType otherwise (legacy needs the matte directly above).
    if (typeof target.setTrackMatte === "function") {
        target.setTrackMatte(matteLayer, type);
        logInfo("matte " + label + ": used setTrackMatte");
    } else {
        target.trackMatteType = type;
        logInfo("matte " + label + ": used legacy trackMatteType");
    }
}
function deselectAll(comp) {
    var i, sel;
    try {
        sel = comp.selectedLayers;
        for (i = 0; i < sel.length; i++) { sel[i].selected = false; }
    } catch (e) {}
}

// ---------------------------------------------------------------------------
// Builders. Each returns nothing; each is called through step().
// ---------------------------------------------------------------------------
function buildSolidsAndTypes() {
    var comp = mkComp("Layer types", ROOT.comps);
    COMPS.types = comp;
    var s1 = solid(comp, "Solid red", COL.red);
    var s2 = solid(comp, "Solid green", COL.green, 800, 600);
    var s3 = solid(comp, "Solid blue", COL.blue, 400, 400);
    var s4 = solid(comp, "Solid yellow", COL.yellow, 300, 300);
    tryq("solid positions", function () {
        xf(s2, "ADBE Position").setValue([600, 400]);
        xf(s3, "ADBE Position").setValue([1300, 600]);
        xf(s4, "ADBE Position").setValue([960, 900]);
    });
    tryq("null", function () { var n = comp.layers.addNull(comp.duration); n.name = "Null"; });
    tryq("adjustment layer", function () {
        var a = solid(comp, "Adjustment", COL.white);
        a.adjustmentLayer = true;
    });
    tryq("guide layer", function () {
        var g = solid(comp, "Guide", COL.grey, 500, 500);
        g.guideLayer = true;
    });
}

function buildShapes() {
    var comp = mkComp("Shapes", ROOT.comps);
    COMPS.shapes = comp;

    tryq("shape rectangle+fill+stroke", function () {
        var l = comp.layers.addShape(); l.name = "Shape rect";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        var c = g.property("ADBE Vectors Group");
        var r = c.addProperty("ADBE Vector Shape - Rect");
        r.property("ADBE Vector Rect Size").setValue([300, 200]);
        r.property("ADBE Vector Rect Roundness").setValue(20);
        var f = c.addProperty("ADBE Vector Graphic - Fill");
        f.property("ADBE Vector Fill Color").setValue([0.9, 0.3, 0.2]);
        var s = c.addProperty("ADBE Vector Graphic - Stroke");
        s.property("ADBE Vector Stroke Color").setValue([1, 1, 1]);
        s.property("ADBE Vector Stroke Width").setValue(8);
        xf(l, "ADBE Position").setValue([400, 300]);
    });
    tryq("shape ellipse+gradient fill+trim paths", function () {
        var l = comp.layers.addShape(); l.name = "Shape ellipse";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        var c = g.property("ADBE Vectors Group");
        var e = c.addProperty("ADBE Vector Shape - Ellipse");
        e.property("ADBE Vector Ellipse Size").setValue([250, 250]);
        c.addProperty("ADBE Vector Graphic - G-Fill");
        var t = c.addProperty("ADBE Vector Filter - Trim");
        t.property("ADBE Vector Trim End").setValueAtTime(0, 0);
        t.property("ADBE Vector Trim End").setValueAtTime(2, 100);
        xf(l, "ADBE Position").setValue([1000, 300]);
    });
    tryq("shape polystar+repeater", function () {
        var l = comp.layers.addShape(); l.name = "Shape polystar";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        var c = g.property("ADBE Vectors Group");
        var st = c.addProperty("ADBE Vector Shape - Star");
        st.property("ADBE Vector Star Points").setValue(7);
        st.property("ADBE Vector Star Outer Radius").setValue(120);
        st.property("ADBE Vector Star Inner Radius").setValue(60);
        var f = c.addProperty("ADBE Vector Graphic - Fill");
        f.property("ADBE Vector Fill Color").setValue([0.2, 0.8, 0.9]);
        var rp = g.property("ADBE Vectors Group").addProperty("ADBE Vector Filter - Repeater");
        rp.property("ADBE Vector Repeater Copies").setValue(4);
        xf(l, "ADBE Position").setValue([400, 750]);
    });
    tryq("shape bezier path+stroke", function () {
        var l = comp.layers.addShape(); l.name = "Shape bezier";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        var c = g.property("ADBE Vectors Group");
        var p = c.addProperty("ADBE Vector Shape - Group");
        p.property("ADBE Vector Shape").setValue(mkShape(
            [[-150, 0], [0, -120], [150, 0], [0, 120]],
            [[0, 60], [-80, 0], [0, -60], [80, 0]],
            [[0, -60], [80, 0], [0, 60], [-80, 0]], true));
        var s = c.addProperty("ADBE Vector Graphic - Stroke");
        s.property("ADBE Vector Stroke Width").setValue(10);
        xf(l, "ADBE Position").setValue([1000, 750]);
    });
    tryq("shape merge paths", function () {
        var l = comp.layers.addShape(); l.name = "Shape merge";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        var c = g.property("ADBE Vectors Group");
        var r = c.addProperty("ADBE Vector Shape - Rect");
        r.property("ADBE Vector Rect Size").setValue([200, 200]);
        var e = c.addProperty("ADBE Vector Shape - Ellipse");
        e.property("ADBE Vector Ellipse Size").setValue([240, 240]);
        var m = c.addProperty("ADBE Vector Filter - Merge");
        m.property("ADBE Vector Merge Type").setValue(3);
        c.addProperty("ADBE Vector Graphic - Fill");
        xf(l, "ADBE Position").setValue([1550, 540]);
    });
}

function buildText() {
    var comp = mkComp("Text", ROOT.comps);
    COMPS.text = comp;
    var t1 = tryq("point text", function () {
        var l = comp.layers.addText("AEPD point text");
        l.name = "Point text";
        xf(l, "ADBE Position").setValue([300, 300]);
        return l;
    });
    tryq("paragraph text", function () {
        var l = comp.layers.addBoxText([800, 400]);
        l.name = "Paragraph text";
        setText(l, "Paragraph text, first line.\rSecond line of the paragraph box.");
        xf(l, "ADBE Position").setValue([300, 500]);
    });
    if (!t1) { logSkip("text animator (no point text layer)"); return; }
    tryq("text animator + range selector", function () {
        var tp = t1.property("ADBE Text Properties");
        var animators = tp.property("ADBE Text Animators");
        animators.addProperty("ADBE Text Animator");
        // Re-fetch after every add: earlier references can go stale.
        var props = animators.property(1).property("ADBE Text Animator Properties");
        tryq("animator position 3D", function () {
            props.addProperty("ADBE Text Position 3D");
            animators.property(1).property("ADBE Text Animator Properties").property("ADBE Text Position 3D").setValue([0, -40, 100]);
        });
        tryq("animator opacity", function () {
            animators.property(1).property("ADBE Text Animator Properties").addProperty("ADBE Text Opacity");
            animators.property(1).property("ADBE Text Animator Properties").property("ADBE Text Opacity").setValue(20);
        });
        // A new animator created by script may have no selector: add a range selector if needed.
        var sels = animators.property(1).property("ADBE Text Selectors");
        if (sels.numProperties === 0) { sels.addProperty("ADBE Text Selector"); }
        sels = animators.property(1).property("ADBE Text Selectors");
        if (sels.numProperties > 0) {
            tryq("range selector offset keys", function () {
                var sel = animators.property(1).property("ADBE Text Selectors").property(1);
                sel.property("ADBE Text Percent Start").setValue(0);
                sel.property("ADBE Text Percent End").setValue(50);
                sel.property("ADBE Text Percent Offset").setValueAtTime(0, -50);
                sel.property("ADBE Text Percent Offset").setValueAtTime(4, 100);
            });
        } else {
            logSkip("range selector could not be added to the text animator");
        }
    });
    tryq("per-character 3D", function () {
        // Unsure of the matchName; try a few candidates, then the menu command. Logged either way.
        var tp = t1.property("ADBE Text Properties");
        var more = tp.property("ADBE Text More Options");
        var cands = ["ADBE Text Per-Character 3D", "ADBE Text Enable Per-char 3D", "ADBE Text Character 3D"];
        var i, done = false, p;
        for (i = 0; i < cands.length && !done; i++) {
            p = null;
            try { p = more.property(cands[i]); } catch (e) { p = null; }
            if (p) { p.setValue(1); done = true; logInfo("per-char 3D set via " + cands[i]); }
        }
        if (!done) {
            var id = app.findMenuCommandId("Enable Per-character 3D");
            if (id) {
                t1.threeDLayer = true;
                comp.openInViewer();
                deselectAll(comp);
                t1.selected = true;
                app.executeCommand(id);
                logInfo("per-char 3D via menu command id " + id);
            } else {
                logSkip("per-character 3D: no known matchName and no menu command");
            }
        }
    });
}

function buildCamerasLights() {
    var comp = mkComp("Cameras and lights", ROOT.comps);
    COMPS.cams = comp;
    tryq("camera one-node", function () {
        var c = comp.layers.addCamera("Camera one-node", [960, 540]);
        c.autoOrient = AutoOrientType.NO_AUTO_ORIENT;
    });
    tryq("camera two-node", function () {
        var c = comp.layers.addCamera("Camera two-node", [960, 540]);
        c.autoOrient = AutoOrientType.CAMERA_OR_POINT_OF_INTEREST;
        tryq("camera depth of field", function () {
            var o = c.property("ADBE Camera Options Group");
            o.property("ADBE Camera Depth of Field").setValue(1);
            o.property("ADBE Camera Aperture").setValue(40);
        });
    });
    var types = [["Light point", LightType.POINT], ["Light spot", LightType.SPOT],
        ["Light ambient", LightType.AMBIENT], ["Light parallel", LightType.PARALLEL]];
    var i;
    for (i = 0; i < types.length; i++) {
        (function (nm, lt) {
            tryq(nm, function () {
                var l = comp.layers.addLight(nm, [960, 540]);
                l.lightType = lt;
                var og = l.property("ADBE Light Options Group");
                var names = ["ADBE Casts Shadows", "ADBE Light Casts Shadows"];
                var k, p = null;
                for (k = 0; k < names.length && !p; k++) {
                    try { p = og.property(names[k]); } catch (e) { p = null; }
                }
                if (lt === LightType.AMBIENT) {
                    logSkip(nm + ": ambient lights have no shadow properties");
                } else if (p) {
                    try { p.setValue(1); } catch (eh) {
                        if (String(eh).indexOf("hidden") >= 0) { logSkip(nm + ": shadows property hidden"); } else { throw eh; }
                    }
                } else { logSkip(nm + ": shadows property not found"); }
            });
        })(types[i][0], types[i][1]);
    }
    tryq("3D layer with material options", function () {
        var l = solid(comp, "3D material", COL.white, 600, 600);
        l.threeDLayer = true;
        var m = l.property("ADBE Material Options Group");
        setIf(m, ["ADBE Casts Shadows"], 1, "Casts Shadows");
        setIf(m, ["ADBE Light Transmission"], 40, "Light Transmission");
        setIf(m, ["ADBE Accepts Shadows"], 0, "Accepts Shadows");
        setIf(m, ["ADBE Accepts Lights"], 0, "Accepts Lights");
        setIf(m, ["ADBE Ambient Coefficient"], 50, "Ambient");
        setIf(m, ["ADBE Diffuse Coefficient"], 60, "Diffuse");
        setIf(m, ["ADBE Specular Coefficient"], 70, "Specular");
        setIf(m, ["ADBE Shininess Coefficient"], 30, "Shininess");
        setIf(m, ["ADBE Metal Coefficient"], 80, "Metal");
    });
}

function buildPrecomps() {
    var l2 = mkComp("Pre level 2", ROOT.pre);
    solid(l2, "L2 solid", COL.green, 600, 600);
    var l1 = mkComp("Pre level 1", ROOT.pre);
    var inner = l1.layers.add(l2);
    inner.name = "Pre L2 layer";
    solid(l1, "L1 solid", COL.blue, 900, 300);
    COMPS.pre1 = l1;
    COMPS.pre2 = l2;
    var main = mkComp("Main precomp nest", ROOT.comps);
    COMPS.nest = main;
    var pl = main.layers.add(l1);
    pl.name = "Pre L1 layer";
    tryq("time remap on precomp", function () {
        pl.timeRemapEnabled = true;
        var tr = pl.property("ADBE Time Remapping");
        tr.setValueAtTime(2.5, 1);
    });
    tryq("time stretch + trims", function () {
        var p2 = main.layers.add(l2);
        p2.name = "Stretched";
        p2.stretch = 150;
        p2.inPoint = 0.5;
        p2.outPoint = 3.5;
    });
}

function buildPlaceholder() {
    var comp = mkComp("Placeholder", ROOT.comps);
    var ph = app.project.importPlaceholder("Placeholder footage", 1280, 720, FPS, 4);
    ph.parentFolder = ROOT.footage;
    comp.layers.add(ph);
}

function buildLayerFeatures() {
    var comp = mkComp("Layer features", ROOT.comps);
    COMPS.feat = comp;
    var base = solid(comp, "Base", COL.grey, 800, 800);

    tryq("masks add/subtract/intersect + feather + expansion", function () {
        var parade = base.property("ADBE Mask Parade");
        var modes = [MaskMode.ADD, MaskMode.SUBTRACT, MaskMode.INTERSECT];
        var i, m;
        for (i = 0; i < 3; i++) {
            m = parade.addProperty("ADBE Mask Atom");
            m.maskMode = modes[i];
            var o = i * 60;
            m.property("ADBE Mask Shape").setValue(mkShape(
                [[100 + o, 100 + o], [700 - o, 100 + o], [700 - o, 700 - o], [100 + o, 700 - o]],
                [[0, 0], [0, 0], [0, 0], [0, 0]], [[0, 0], [0, 0], [0, 0], [0, 0]], true));
            m.property("ADBE Mask Feather").setValue([10 * (i + 1), 10 * (i + 1)]);
            m.property("ADBE Mask Offset").setValue(5 * i);
            if (i === 0) {
                // animated mask path
                var shp = m.property("ADBE Mask Shape");
                shp.setValueAtTime(0, shp.value);
                shp.setValueAtTime(3, mkShape(
                    [[50, 50], [750, 120], [650, 750], [120, 700]],
                    [[0, 0], [0, 0], [0, 0], [0, 0]], [[0, 0], [0, 0], [0, 0], [0, 0]], true));
            }
        }
    });

    tryq("blending modes", function () {
        var modes = [BlendingMode.MULTIPLY, BlendingMode.SCREEN, BlendingMode.OVERLAY, BlendingMode.ADD];
        var i, l;
        for (i = 0; i < modes.length; i++) {
            l = solid(comp, "Blend " + i, COL.blue, 200, 200);
            l.blendingMode = modes[i];
            xf(l, "ADBE Position").setValue([300 + i * 250, 900]);
        }
    });

    tryq("motion blur", function () {
        var l = solid(comp, "Motion blur", COL.red, 150, 150);
        l.motionBlur = true;
        comp.motionBlur = true;
        var p = xf(l, "ADBE Position");
        p.setValueAtTime(0, [100, 200]);
        p.setValueAtTime(1, [1800, 200]);
    });

    tryq("parenting chain", function () {
        var a = solid(comp, "Parent A", COL.yellow, 100, 100);
        var b = solid(comp, "Parent B", COL.green, 100, 100);
        var c = solid(comp, "Parent C", COL.red, 100, 100);
        c.parent = b;
        b.parent = a;
    });

    tryq("labels", function () {
        var i, l;
        for (i = 1; i <= 4; i++) {
            l = solid(comp, "Label " + i, COL.white, 50, 50);
            l.label = i * 3;
        }
    });

    tryq("layer markers", function () {
        var l = solid(comp, "Markers", COL.white, 100, 100);
        var mp = l.property("ADBE Marker");
        var m1 = newMarker("Layer marker comment");
        m1.chapter = "Chapter one";
        m1.url = "https://example.invalid/";
        m1.frameTarget = "_blank";
        m1.cuePointName = "cue_one";
        m1.duration = 1;
        tryq("marker setParameters", function () { m1.setParameters({ keyA: "valueA", keyB: "valueB" }); });
        mp.setValueAtTime(1, m1);
        var m2 = newMarker("Second marker");
        mp.setValueAtTime(3, m2);
    });
    tryq("comp markers", function () {
        var m = newMarker("Comp marker");
        m.chapter = "Comp chapter";
        m.duration = 0.5;
        comp.markerProperty.setValueAtTime(2, m);
    });

    // Flags go last: locking a layer prevents later changes to it.
    tryq("shy/solo/locked flags", function () {
        var s = solid(comp, "Flag shy", COL.white, 60, 60);
        s.shy = true;
        var so = solid(comp, "Flag solo", COL.white, 60, 60);
        so.solo = true;
        var lk = solid(comp, "Flag locked", COL.white, 60, 60);
        lk.locked = true;
    });
}

function buildMattes() {
    var comp = mkComp("Track mattes", ROOT.comps);
    COMPS.mattes = comp;
    var defs = [["alpha", TrackMatteType.ALPHA], ["alpha inverted", TrackMatteType.ALPHA_INVERTED],
        ["luma", TrackMatteType.LUMA]];
    var i;
    for (i = 0; i < defs.length; i++) {
        (function (nm, ty, idx) {
            tryq("matte " + nm, function () {
                // target first, matte after: new layers go on top, so the matte ends directly above.
                var tgt = solid(comp, "Target " + nm, COL.blue, 500, 500);
                var mt = solid(comp, "Matte " + nm, COL.white, 300, 300);
                xf(tgt, "ADBE Position").setValue([400 + idx * 500, 540]);
                xf(mt, "ADBE Position").setValue([400 + idx * 500, 540]);
                matte(comp, tgt, mt, ty, nm);
            });
        })(defs[i][0], defs[i][1], i);
    }
    if (typeof comp.layers.addSolid === "function") {
        var comp2 = mkComp("Track matte shared", ROOT.comps);
        COMPS.matteShared = comp2;
        tryq("one matte for two layers, matte not adjacent (AE 23+)", function () {
            var probe = solid(comp2, "probe", COL.white, 10, 10);
            var hasNew = (typeof probe.setTrackMatte === "function");
            probe.remove();
            if (!hasNew) { logSkip("shared track matte: setTrackMatte not available in this AE"); return; }
            var m = solid(comp2, "Shared matte", COL.white, 600, 600);      // bottom
            solid(comp2, "Spacer 1", COL.grey, 100, 100);
            solid(comp2, "Spacer 2", COL.grey, 100, 100);
            var a = solid(comp2, "Uses matte A", COL.red, 900, 900);
            var b = solid(comp2, "Uses matte B", COL.green, 900, 900);      // top
            a.setTrackMatte(m, TrackMatteType.ALPHA);
            b.setTrackMatte(m, TrackMatteType.ALPHA_INVERTED);
        });
    }
}

function buildKeyframes() {
    var comp = mkComp("Keyframes", ROOT.comps);
    COMPS.kf = comp;
    tryq("linear opacity", function () {
        var l = solid(comp, "KF linear", COL.red, 200, 200);
        var p = xf(l, "ADBE Opacity");
        p.setValueAtTime(0, 0); p.setValueAtTime(2, 100);
        p.setInterpolationTypeAtKey(1, KeyframeInterpolationType.LINEAR);
        p.setInterpolationTypeAtKey(2, KeyframeInterpolationType.LINEAR);
    });
    tryq("bezier custom ease opacity", function () {
        var l = solid(comp, "KF bezier ease", COL.green, 200, 200);
        var p = xf(l, "ADBE Opacity");
        p.setValueAtTime(0, 0); p.setValueAtTime(2, 100); p.setValueAtTime(4, 20);
        p.setInterpolationTypeAtKey(2, KeyframeInterpolationType.BEZIER, KeyframeInterpolationType.BEZIER);
        p.setTemporalEaseAtKey(2, [new KeyframeEase(10, 80)], [new KeyframeEase(60, 25)]);
    });
    tryq("hold rotation", function () {
        var l = solid(comp, "KF hold", COL.blue, 200, 200);
        var p = xfAny(l, ["ADBE Rotate Z", "ADBE Rotation"]);
        p.setValueAtTime(0, 0); p.setValueAtTime(1, 90); p.setValueAtTime(2, 180);
        p.setInterpolationTypeAtKey(1, KeyframeInterpolationType.HOLD);
        p.setInterpolationTypeAtKey(2, KeyframeInterpolationType.HOLD);
    });
    tryq("spatial bezier + roving position", function () {
        var l = solid(comp, "KF spatial roving", COL.yellow, 150, 150);
        var p = xf(l, "ADBE Position");
        p.setValueAtTime(0, [200, 800]);
        p.setValueAtTime(1, [600, 300]);
        p.setValueAtTime(2.5, [1000, 900]);
        p.setValueAtTime(4, [1700, 300]);
        tryq("spatial tangents", function () {
            try {
                p.setSpatialTangentsAtKey(2, [200, -100], [-200, 100]);
            } catch (e) {
                p.setSpatialTangentsAtKey(2, [200, -100, 0], [-200, 100, 0]);
            }
        });
        tryq("roving", function () {
            p.setRovingAtKey(2, true);
            p.setRovingAtKey(3, true);
        });
    });
    tryq("separated position dimensions", function () {
        var l = solid(comp, "KF separated", COL.white, 150, 150);
        var p = xf(l, "ADBE Position");
        p.dimensionsSeparated = true;
        var px = xf(l, "ADBE Position_0");
        var py = xf(l, "ADBE Position_1");
        px.setValueAtTime(0, 100); px.setValueAtTime(3, 1500);
        py.setValueAtTime(0, 900); py.setValueAtTime(3, 200);
    });
}

function buildExpressions() {
    var comp = mkComp("Expressions", ROOT.comps);
    COMPS.expr = comp;
    var follower = solid(comp, "Follower", COL.red, 150, 150);
    var leader = solid(comp, "Leader", COL.green, 150, 150);
    var ctl = solid(comp, "Slider holder", COL.grey, 100, 100);
    var wig = solid(comp, "Wiggler", COL.blue, 150, 150);
    tryq("wiggle", function () { xf(wig, "ADBE Position").expression = "wiggle(2, 40)"; });
    tryq("link by layer name", function () {
        xf(follower, "ADBE Position").expression = "thisComp.layer(\"Leader\").transform.position + [200, 0]";
        xf(leader, "ADBE Position").setValueAtTime(0, [300, 300]);
        xf(leader, "ADBE Position").setValueAtTime(4, [900, 600]);
    });
    tryq("thisComp.layer(index + 1)", function () {
        xf(wig, "ADBE Opacity").expression = "thisComp.layer(index + 1).transform.opacity * 0.5";
    });
    tryq("slider-controlled expression", function () {
        var e = ctl.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        e.name = "Amount";
        e.property(1).setValue(25);
        xfAny(wig, ["ADBE Rotate Z", "ADBE Rotation"]).expression = "thisComp.layer(\"Slider holder\").effect(\"Amount\")(\"Slider\") * time * 4";
    });
}

function addFx(layer, matchName, label) {
    // Add one effect by matchName in its own try/catch. Returns the effect or null.
    try {
        var parade = layer.property("ADBE Effect Parade");
        var e = parade.addProperty(matchName);
        if (!e) { logFail("fx " + label + " (" + matchName + ")", "addProperty returned null"); return null; }
        return e;
    } catch (ex) {
        logFail("fx " + label + " (" + matchName + ")", ex);
        return null;
    }
}

function buildFxCore() {
    var comp = mkComp("FX core", ROOT.fx);
    COMPS.fxcore = comp;
    var list = [
        ["Gaussian Blur", "ADBE Gaussian Blur 2", "key"],
        ["Fast Box Blur", "ADBE Box Blur2", "val"],
        ["Fill", "ADBE Fill", ""],
        ["Tint", "ADBE Tint", ""],
        ["Curves", "ADBE CurvesCustom", ""],
        ["Levels", "ADBE Easy Levels2", ""],
        ["Hue/Saturation", "ADBE HUE SATURATION", ""],
        ["Fractal Noise", "ADBE Fractal Noise", "key"],
        ["Drop Shadow", "ADBE Drop Shadow", "val"],
        ["Glow", "ADBE Glo2", ""],
        ["Turbulent Displace", "ADBE Turbulent Displace", ""],
        ["Gradient Ramp", "ADBE Ramp", ""],
        ["4-Color Gradient", "ADBE 4ColorGradient", ""],
        ["Stroke", "ADBE Stroke", ""],
        ["Echo", "ADBE Echo", ""],
        ["Transform", "ADBE Geometry2", ""],
        ["Motion Tile", "ADBE Tile", ""],
        ["Offset", "ADBE Offset", ""],
        ["Set Matte", "ADBE Set Matte3", ""],
        ["Roughen Edges", "ADBE Roughen Edges", ""],
        ["Simple Choker", "ADBE Simple Choker", ""],
        ["CC Particle World", "CC Particle World", ""],
        ["Venetian Blinds", "ADBE Venetian Blinds", ""],
        ["Lumetri Color", "ADBE Lumetri", ""],
        ["Slider Control", "ADBE Slider Control", "key"],
        ["Angle Control", "ADBE Angle Control", ""],
        ["Checkbox Control", "ADBE Checkbox Control", ""],
        ["Color Control", "ADBE Color Control", ""],
        ["Point Control", "ADBE Point Control", ""],
        ["3D Point Control", "ADBE Point3D Control", ""],
        ["Layer Control", "ADBE Layer Control", ""],
        ["Dropdown Menu Control", "ADBE Dropdown Control", ""]
    ];
    var perSolid = 6;
    var i, layer = null, e, n = 0;
    for (i = 0; i < list.length; i++) {
        if (i % perSolid === 0) {
            n++;
            layer = solid(comp, "FX core " + n, COL.grey);
        }
        e = addFx(layer, list[i][1], list[i][0]);
        if (e) {
            logOk("fx " + list[i][0]);
            if (list[i][2] === "val" || list[i][2] === "key") {
                (function (ee, nm, mode) {
                    tryq("fx param " + nm, function () {
                        var p = ee.property(mode === "key" ? 1 : 2);
                        var v = p.value;
                        if (typeof v === "number") { p.setValue(nm === "Gaussian Blur" ? 25 : v + 1); }
                    });
                    if (mode === "key") {
                        tryq("fx keyframe " + nm, function () {
                            var p = ee.property(1);
                            var v = p.value;
                            p.setValueAtTime(0, v);
                            p.setValueAtTime(2, (typeof v === "number") ? v + 10 : v);
                        });
                    }
                })(e, list[i][0], list[i][2]);
            }
        }
    }
    return comp;
}

function shouldSkipEffect(eff) {
    var mn = String(eff.matchName);
    var dn = "";
    try { dn = String(eff.displayName).toLowerCase(); } catch (e) {}
    var lmn = mn.toLowerCase();
    var i;
    if (arrHas(SKIP_EFFECT_MATCHNAMES, mn)) { return true; }
    if (arrHas(SKIP_EFFECT_MATCHNAMES_DIALOG, mn)) { return true; }
    for (i = 0; i < SKIP_EFFECT_SUBSTRINGS.length; i++) {
        if (lmn.indexOf(SKIP_EFFECT_SUBSTRINGS[i]) >= 0 || dn.indexOf(SKIP_EFFECT_SUBSTRINGS[i]) >= 0) { return true; }
    }
    return false;
}

function buildFxAll() {
    var comp = mkComp("FX all", ROOT.fx);
    COMPS.fxall = comp;
    var list = [];
    var effs = app.effects;
    var i, e, mn;
    for (i = 0; i < effs.length; i++) {
        e = effs[i];
        if (!e || !e.matchName) { continue; }
        mn = String(e.matchName);
        if (mn.indexOf("ADBE ") === 0 || mn.indexOf("CC ") === 0) { list.push(e); }
    }
    list.sort(function (a, b) { return (a.matchName < b.matchName) ? -1 : ((a.matchName > b.matchName) ? 1 : 0); });
    logInfo("FX all: " + list.length + " candidate effects of " + effs.length);
    var layer = null, count = 0, n = 0, added = 0, failed = 0, skipped = 0, slow = 0;
    var t0, dt;
    for (i = 0; i < list.length; i++) {
        e = list[i];
        mn = String(e.matchName);
        if (shouldSkipEffect(e)) { skipped++; logSkip("FX all skip list: " + mn); continue; }
        try {
            if (!layer || count >= EFFECTS_PER_SOLID) {
                n++;
                if (layer) { logInfo("FX all layer '" + layer.name + "': " + count + " effects applied"); }
                layer = solid(comp, "FX all " + (n < 10 ? "0" : "") + n, COL.grey, 640, 360);
                count = 0;
            }
            var parade = layer.property("ADBE Effect Parade");
            var ok = true;
            try { if (parade.canAddProperty(mn) === false) { ok = false; } } catch (e0) { ok = true; }
            if (!ok) { failed++; FAILS++; LOG.push("FAIL  FX all cannot add: " + mn); continue; }
            t0 = new Date().getTime();
            var p = parade.addProperty(mn);
            dt = new Date().getTime() - t0;
            if (dt > SLOW_EFFECT_MS) { slow++; LOG.push("SLOW  " + mn + " " + dt + " ms (possible dialog)"); }
            if (!p) { failed++; FAILS++; LOG.push("FAIL  FX all addProperty null: " + mn); continue; }
            added++; count++;
        } catch (ex) {
            failed++;
            logFail("FX all " + mn, ex);
        }
    }
    if (layer) { logInfo("FX all layer '" + layer.name + "': " + count + " effects applied"); }
    logInfo("FX all: slow effects " + slow + ", added " + added + ", failed " + failed + ", skipped " + skipped + ", solids " + n);
}

function buildEssential() {
    var comp = mkComp("Essential Graphics", ROOT.comps);
    COMPS.eg = comp;
    if (!has(comp, "motionGraphicsTemplateName")) { logSkip("Essential Graphics: API not present"); return; }
    var ctl = solid(comp, "EG controls", COL.grey, 300, 300);
    var parade = ctl.property("ADBE Effect Parade");
    var e1 = parade.addProperty("ADBE Slider Control"); e1.name = "EG Slider";
    e1 = null;
    var e2 = parade.addProperty("ADBE Color Control"); e2.name = "EG Color";
    e2 = null;
    var e3 = parade.addProperty("ADBE Checkbox Control"); e3.name = "EG Checkbox";
    e3 = null;
    comp.motionGraphicsTemplateName = "AEPD Test Template";
    // Re-fetch all references now: adding more effects invalidates earlier ones.
    parade = ctl.property("ADBE Effect Parade");
    var arr = [];
    var nms = ["EG Slider", "EG Color", "EG Checkbox"];
    var q, ef;
    for (q = 0; q < nms.length; q++) {
        ef = parade.property(nms[q]);
        if (ef) { arr.push(ef.property(1)); } else { logSkip("EG control not found: " + nms[q]); }
    }
    var i, added = 0;
    for (i = 0; i < arr.length; i++) {
        try {
            if (typeof arr[i].addToMotionGraphicsTemplate === "function") {
                if (typeof arr[i].canAddToMotionGraphicsTemplate === "function" &&
                    !arr[i].canAddToMotionGraphicsTemplate(comp)) {
                    logSkip("EG property " + i + ": cannot add");
                    continue;
                }
                arr[i].addToMotionGraphicsTemplate(comp);
                added++;
            } else {
                logSkip("addToMotionGraphicsTemplate not available");
            }
        } catch (e) { logFail("EG add property " + i, e); }
    }
    logInfo("Essential Graphics: added " + added + " properties");
}

function buildLayerStyles() {
    var comp = mkComp("Layer styles", ROOT.comps);
    COMPS.styles = comp;
    var cmds = ["Drop Shadow", "Inner Shadow", "Outer Glow", "Stroke"];
    var i, id, l;
    try { comp.openInViewer(); } catch (e) { logFail("openInViewer for layer styles", e); }
    for (i = 0; i < cmds.length; i++) {
        id = 0;
        try { id = app.findMenuCommandId(cmds[i]); } catch (e2) { id = 0; }
        if (!id) { continue; } // skip silently
        try {
            l = solid(comp, "Style " + cmds[i], COL.blue, 400, 400);
            deselectAll(comp);
            l.selected = true;
            app.executeCommand(id);
            logOk("layer style command '" + cmds[i] + "' id " + id);
        } catch (e3) { logFail("layer style " + cmds[i], e3); }
    }
}

function buildNewFeatures() {
    // Feature detection only. Nothing is guessed: a missing API is a logged skip.
    tryq("ThreeDModelLayer probe", function () {
        if (typeof ThreeDModelLayer !== "undefined") {
            logInfo("ThreeDModelLayer class exists (no model file is imported by this script)");
        } else {
            logSkip("ThreeDModelLayer not available in " + app.version);
        }
    });
    tryq("renderers", function () {
        var probe = mkComp("Renderers", ROOT.comps);
        var names = null;
        try { names = probe.renderers; } catch (e) { names = null; }
        if (!names) { logSkip("comp.renderers not available"); return; }
        logInfo("renderers: " + names.join(" | "));
        var i;
        for (i = 0; i < names.length; i++) {
            (function (idx, nm) {
                tryq("renderer " + nm, function () {
                    var c = mkComp("Renderer " + (idx + 1), ROOT.comps);
                    var s = solid(c, "3D solid", COL.white, 400, 400);
                    s.threeDLayer = true;
                    c.renderer = nm;
                    logInfo("renderer set: " + c.renderer);
                });
            })(i, names[i]);
        }
    });
    tryq("expression engine", function () {
        if (has(app.project, "expressionEngine")) { logInfo("expressionEngine = " + app.project.expressionEngine); }
        else { logSkip("project.expressionEngine not available"); }
    });
    tryq("working color space", function () {
        if (has(app.project, "workingSpace")) { logInfo("workingSpace = " + app.project.workingSpace + " (left default)"); }
        else { logSkip("project.workingSpace not available (pre-22?)"); }
    });
    logSkip("variable-font axes: no API known with confidence; not attempted");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
function main() {
    var v = verParts();
    var tag = v.major + "." + v.minor + "_" + osTag();

    if (!fileAccessOn()) {
        aepdAlert("AEPD test project: scripts are not allowed to write files.\n\n" +
            "Turn it on: Edit > Preferences > Scripting & Expressions (on Mac: After Effects > Settings > Scripting & Expressions), " +
            "tick \"Allow Scripts to Write Files and Access Network\", then run this script again.");
        return;
    }
    if (app.project && app.project.numItems > 0) {
        aepdAlert("AEPD test project: a project with content is open.\n\nPlease close it (File > Close Project, do not save) and run the script again. " +
            "This script never saves over your work.");
        return;
    }
    var folder = aepdPickFolder("AEPD test project: choose the folder for the output");
    if (!folder) { return; }

    var projName = "AEPD_test_" + tag + ".aep";
    var projPath = folder.fsName + "/" + projName;
    var logPath = folder.fsName + "/make_test_project_log_" + tag + ".txt";

    var RUN_START = new Date().getTime();
    LOG.push("AEPD make_test_project");
    LOG.push("app.version = " + app.version);
    try { LOG.push("app.buildName = " + app.buildName); } catch (e0) {}
    LOG.push("$.os = " + $.os);
    LOG.push("INCLUDE_ALL_EFFECTS = " + INCLUDE_ALL_EFFECTS);
    LOG.push("");

    app.beginUndoGroup("AEPD test project");
    try {
        step("folders", function () {
            ROOT.comps = mkFolder("01 Comps");
            ROOT.pre = mkFolder("02 Precomps");
            ROOT.footage = mkFolder("03 Footage");
            ROOT.fx = mkFolder("04 Effects");
        });
        step("project bit depth 16", function () { app.project.bitsPerChannel = 16; });

        step("layer types: solids/null/adjustment/guide", buildSolidsAndTypes);
        step("shape layers", buildShapes);
        step("text", buildText);
        step("cameras, lights, 3D material", buildCamerasLights);
        step("precomps, time remap, stretch", buildPrecomps);
        step("placeholder footage", buildPlaceholder);
        step("layer features", buildLayerFeatures);
        step("track mattes", buildMattes);
        step("keyframes", buildKeyframes);
        step("expressions", buildExpressions);
        step("FX core", buildFxCore);
        if (INCLUDE_ALL_EFFECTS) { step("FX all", buildFxAll); } else { logSkip("FX all disabled"); }
        step("essential graphics", buildEssential);
        step("layer styles via menu commands", buildLayerStyles);
        step("newer-version feature probes", buildNewFeatures);
    } catch (eAll) {
        logFail("unexpected top-level error", eAll);
    }
    try { app.endUndoGroup(); } catch (eU) {}

    var saved = false;
    try {
        app.project.save(new File(projPath));
        saved = true;
        logOk("saved " + projPath);
    } catch (eS) { logFail("save project", eS); }

    var labels = ["Save a Copy As 26.x...", "Save a Copy As 25.x...", "Save a Copy As 24.x...",
        "Save a Copy As 23.x...", "Save a Copy As 22.x...", "Save a Copy As 18.x...", "Save a Copy As 17.x..."];
    var i, id;
    for (i = 0; i < labels.length; i++) {
        id = 0;
        try { id = app.findMenuCommandId(labels[i]); } catch (eL) { id = 0; }
        LOG.push("BACKWARD " + labels[i] + " -> command id " + id);
        if (id) { BACKWARD.push(labels[i].replace("Save a Copy As ", "").replace("...", "")); }
    }
    // Not auto-run: it is not known that executeCommand returns only after the dialog closes.

    LOG.push("");
    LOG.push("Total run time: " + Math.round((new Date().getTime() - RUN_START) / 1000) + " s");
    LOG.push("Summary: failures=" + FAILS + ", skipped=" + SKIPS + ", saved=" + saved);
    var wrote = writeText(logPath, LOG.join("\n") + "\n");

    var msg = "AEPD test project\n\n";
    msg += saved ? ("Saved: " + projPath + "\n") : "The project could NOT be saved (see log).\n";
    msg += "Problems logged: " + FAILS + "   Skipped features: " + SKIPS + "\n";
    msg += wrote ? ("Log: " + logPath + "\n") : "Log file could not be written.\n";
    msg += "\nOlder-version Save a Copy As targets found in this After Effects:\n";
    msg += BACKWARD.length ? ("  " + BACKWARD.join(", ") + "\n") : "  none found by name (see README, use the File menu manually)\n";
    msg += "\nNext: File > Save a Copy As > pick each older version, name the files as described in the README. " +
        "Send the whole folder back (even if there were problems).";
    aepdAlert(msg);
}

main();
