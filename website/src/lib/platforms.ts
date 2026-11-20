/**
 * The platforms a release ships for, as built by `.github/workflows/release.yml` ("Collect"
 * step). Asset names are `open-camera-mouse-<version>-<suffix>`; the suffixes here are what
 * the OS-aware download (WEBSITE_ROADMAP.md Phase 4) will match against. Requirements and
 * caveats come from README.md ("Install", "Known limitations"); keep them in sync.
 */
export type PlatformId = "macos" | "windows" | "linux";

export interface Platform {
  id: PlatformId;
  name: string;
  /** Who the build is for, in one line. */
  requirement: string;
  /** Release asset suffixes, preferred first. */
  assets: { suffix: string; label: string }[];
  /** README.md install section for this platform. */
  readmeAnchor: string;
  /** Known limitations worth knowing before downloading (README "Known limitations"). */
  notes: string[];
}

export const PLATFORMS: Platform[] = [
  {
    id: "macos",
    name: "macOS",
    requirement: "Apple Silicon (M-series) Macs",
    assets: [{ suffix: "macos-arm64.dmg", label: "Disk image (.dmg)" }],
    readmeAnchor: "macos-apple-silicon",
    notes: ["Intel Macs aren't supported.", "Not notarized yet: macOS blocks the first launch."],
  },
  {
    id: "windows",
    name: "Windows",
    requirement: "64-bit Windows on x64",
    assets: [{ suffix: "windows-x64-setup.exe", label: "Installer (.exe)" }],
    readmeAnchor: "windows-x64",
    notes: [
      "Unsigned: SmartScreen shows a warning.",
      "Not yet tested on a real Windows device.",
    ],
  },
  {
    id: "linux",
    name: "Linux",
    requirement: "x86_64 with an X11 session",
    assets: [
      { suffix: "linux-x86_64.AppImage", label: "AppImage" },
      { suffix: "linux-x86_64.tar.gz", label: "Archive (.tar.gz)" },
    ],
    readmeAnchor: "linux-x86_64-x11",
    notes: ["Wayland isn't supported.", "Not yet tested on a real Linux device."],
  },
];
