import { describe, expect, it } from "vitest";
import { redactAll, redactText } from "./redact";

const NAMES = ["jane", "doe", "acme", "kristina", "müller", "иванов", "проект", "clients"];
/** Nothing that looks like a user name, folder name or the home directory may survive. */
function expectNoLeak(output: string, forbidden: string[] = NAMES) {
  for (const word of forbidden) expect(output.toLowerCase()).not.toContain(word.toLowerCase());
  expect(output).not.toMatch(/Users|home\/|Volumes|[A-Za-z]:\\/);
}

describe("redactText: path shapes", () => {
  it("handles Windows paths with a drive letter and backslashes", () => {
    const out = redactText(String.raw`Added C:\Users\Jane\Clients\Acme\Intro_v05.aep`);
    expect(out).toBe("Added <file-1>.aep");
    expectNoLeak(out);
  });

  it("handles forward slashes and a lower-case drive", () => {
    expect(redactText("d:/Projects/Acme/x.aep")).toBe("<file-1>.aep");
  });

  it("handles names with spaces without being told about them", () => {
    const out = redactText(String.raw`C:\Users\Jane Doe\My Clients\Acme Corp\Final Cut v2.aep failed`);
    expect(out).toBe("<file-1>.aep failed");
    expectNoLeak(out);
  });

  it("handles UNC paths", () => {
    const out = redactText(String.raw`read \\FILESERVER\Projects\Acme\Intro.aep failed`);
    expect(out).toBe("read <file-1>.aep failed");
    expectNoLeak(out, ["fileserver", "acme", "projects"]);
  });

  it("handles macOS and Linux home paths", () => {
    const mac = redactText("/Users/jane/Movies/Acme/Intro_v05.aep");
    const linux = redactText("/home/jane/Dev/acme/Intro_v05.aep");
    const volume = redactText("/Volumes/Work Drive/Acme/Intro.aep");
    expect([mac, linux, volume]).toEqual(["<file-1>.aep", "<file-1>.aep", "<file-1>.aep"]);
  });

  it("handles ~ and environment-variable prefixes and file URLs", () => {
    expect(redactText("~/Documents/Acme/x.aep")).toBe("<file-1>.aep");
    expect(redactText(String.raw`%USERPROFILE%\Desktop\x.aep`)).toBe("<file-1>.aep");
    expect(redactText("file:///home/jane/Acme/x.aep")).toBe("<file-1>.aep");
    expect(redactText("file:///C:/Users/Jane/x.aep")).toBe("<file-1>.aep");
  });

  it("handles non-ASCII folder names", () => {
    const out = redactText(String.raw`C:\Users\Иванов\Проекты\Müller GmbH\Ролик_v2.aep`);
    expect(out).toBe("<file-1>.aep");
    expectNoLeak(out);
    expect(redactText("/home/müller/проект/клиент/Intro.aep")).toBe("<file-1>.aep");
  });

  it("handles the escaped backslashes of JSON error text", () => {
    const out = redactText(String.raw`{"path":"C:\\Users\\jane\\Acme\\Intro.aep"}`);
    expectNoLeak(out);
    expect(out).toContain("<file-1>.aep");
  });

  it("keeps the extension, lower-cased, and drops it for folders", () => {
    expect(redactText("/a/b/C.AEP")).toBe("<file-1>.aep");
    expect(redactText("/a/b/Glow.ffx")).toBe("<file-1>.ffx");
    expect(redactText("/home/jane/Converted", { folders: ["/home/jane/Converted"] })).toBe("<folder-1>");
    expect(redactText("/home/jane/Converted")).toBe("<path-1>");
  });
});

describe("redactText: inside messages and technical detail", () => {
  it("keeps the surrounding sentence and punctuation", () => {
    expect(redactText("Could not write /home/jane/acme/out.aep.")).toBe("Could not write <file-1>.aep.");
    expect(redactText('open("/home/jane/acme/out.aep") failed')).toBe('open("<file-1>.aep") failed');
    expect(redactText("(see /home/jane/acme/out.aep), then retry")).toBe("(see <file-1>.aep), then retry");
  });

  it("redacts paths in OS error text", () => {
    const out = redactText(String.raw`Permission denied (os error 13) at C:\Users\Jane\AppData\Local\Temp\.Intro.aep.4321.tmp`);
    expectNoLeak(out, ["jane", "appdata", "temp"]);
    expect(out).toMatch(/^Permission denied \(os error 13\) at <file-1>\.tmp$/);
  });

  it("redacts two paths in one line separately", () => {
    const out = redactText("copy /home/jane/a/One.aep to /home/jane/b/Two.aep");
    expect(out).toBe("copy <file-1>.aep to <file-2>.aep");
    expect(redactText(String.raw`copy C:\a\One.aep to D:\b\Two.aep`)).toBe("copy <file-1>.aep to <file-2>.aep");
  });

  it("redacts key=value technical detail", () => {
    const out = redactText("output=/home/jane/Acme/Intro_AE2024.aep size=12 MB");
    expect(out).toBe("output=<file-1>.aep size=12 MB");
  });

  it("scrubs a home directory fragment left without a root", () => {
    const out = redactText(String.raw`...\Users\jane\AppData\x`);
    expectNoLeak(out, ["jane"]);
  });

  it("leaves ordinary text, URLs, versions and dates alone", () => {
    for (const text of [
      "Checked 3 files for After Effects 2024 (24.5, build 52)",
      "https://github.com/itsAnchorpoint/AEP-Downgrader/releases/latest",
      "on/off and/or 10/12/2026 at 12:30",
      "Detected 3 / 5, v2.0.0-alpha.0",
      "shadow-color-removed (2), effects-removed (3)",
    ]) {
      expect(redactText(text)).toBe(text);
    }
  });
});

describe("redactAll: tags", () => {
  it("gives the same file the same tag everywhere, across texts and separators", () => {
    const [a, b, c] = redactAll([
      String.raw`Added C:\Users\Jane\Acme\Intro.aep`,
      "Failed c:/users/jane/acme/intro.aep",
      String.raw`Done C:\Users\Jane\Acme\Intro.aep`,
    ]);
    expect(a).toBe("Added <file-1>.aep");
    expect(b).toBe("Failed <file-1>.aep");
    expect(c).toBe("Done <file-1>.aep");
  });

  it("gives two files with the same name in different folders different tags", () => {
    const out = redactAll(["/home/jane/acme/Intro.aep", "/home/jane/other/Intro.aep", "/home/jane/acme/Intro.aep"]);
    expect(out).toEqual(["<file-1>.aep", "<file-2>.aep", "<file-1>.aep"]);
  });

  it("numbers tags in order of first appearance", () => {
    expect(redactAll(["/a/x.aep /b/y.ffx", "/c/z.aep"])).toEqual(["<file-1>.aep <file-2>.ffx", "<file-3>.aep"]);
  });

  it("is deterministic", () => {
    const input = ["/a/x.aep", "/b/x.aep"];
    expect(redactAll(input)).toEqual(redactAll(input));
  });
});

describe("redactAll: known paths", () => {
  const known = {
    files: [String.raw`C:\Users\Jane Doe\Work Stuff\Final v2 (copy) - Acme.aep`, "/home/jane/My Projects/Intro 01.aep"],
    folders: ["/home/jane/Out Folder"],
  };

  it("redacts known paths with spaces and odd characters literally", () => {
    const [out] = redactAll([String.raw`Added C:\Users\Jane Doe\Work Stuff\Final v2 (copy) - Acme.aep for 2024`], known);
    expect(out).toBe("Added <file-1>.aep for 2024");
    expectNoLeak(out);
  });

  it("shares the tag between a known path and the same path found generically", () => {
    const out = redactAll(["a /home/jane/My Projects/Intro 01.aep b", "again /home/jane/My Projects/Intro 01.aep"], known);
    expect(out).toEqual(["a <file-1>.aep b", "again <file-1>.aep"]);
  });

  it("treats what follows a known folder as a separate path", () => {
    const [out] = redactAll(["wrote /home/jane/Out Folder/Intro_AE2024.aep in /home/jane/Out Folder"], known);
    expect(out).toBe("wrote <file-1>.aep in <folder-1>");
    expectNoLeak(out);
  });

  it("prefers the longest known path", () => {
    const [out] = redactAll(["/a/b/c.aep"], { files: ["/a/b/c.aep"], folders: ["/a/b"] });
    expect(out).toBe("<file-1>.aep");
  });

  it("ignores empty entries", () => {
    expect(redactAll(["hello"], { files: [""], folders: [""] })).toEqual(["hello"]);
  });
});

describe("no user name survives", () => {
  const user = "Kristina";
  const samples = [
    String.raw`C:\Users\${user}\Documents\Acme\x.aep`,
    String.raw`C:\Users\${user}\AppData\Roaming\AEP Downgrader\settings.json`,
    `/Users/${user}/Library/Application Support/x.aep`,
    `/home/${user.toLowerCase()}/.config/aep/settings.json`,
    String.raw`\\NAS-${user}\share\x.aep`,
    `~${user}/x.aep`,
    String.raw`error reading C:\\Users\\${user}\\x.aep: not found`,
    `file:///home/${user}/x.aep`,
    `cannot create /home/${user}/Converted/Out.aep (Permission denied)`,
  ];
  for (const sample of samples) {
    it(sample, () => {
      expect(redactText(sample.replace("${user}", user))).not.toMatch(/kristina/i);
    });
  }
});
