/**
 * The platforms a release ships for, as built by `.github/workflows/release.yml` ("Collect"
 * step). Asset names are `open-camera-mouse-<version>-<suffix>`; the suffixes here are what
 * the OS-aware download (docs/WEBSITE.md "Downloads") matches against. Requirements and
 * caveats come from README.md ("Install", "Known limitations"); keep them in sync.
 */
export type PlatformId = "macos" | "windows" | "linux";

export interface Platform {
  id: PlatformId;
  name: string;
  /** Who the build is for, in one line. */
  requirement: string;
  /** The same in a word or two, for the hero line (`v0.4.0 · Apple Silicon`). */
  short: string;
  /** Release asset suffixes, preferred first. */
  assets: { suffix: string; label: string }[];
  /** Heading id of this platform's section in the manual's Installation page (build checks it). */
  installAnchor: string;
  /** Known limitations worth knowing before downloading (README "Known limitations"). */
  notes: string[];
}

export const PLATFORMS: Platform[] = [
  {
    id: "macos",
    name: "macOS",
    requirement: "Apple Silicon (M-series) Macs",
    short: "Apple Silicon",
    assets: [{ suffix: "macos-arm64.dmg", label: "Disk image (.dmg)" }],
    installAnchor: "macos-apple-silicon",
    notes: ["Intel Macs aren't supported.", "Not notarized: allow the first launch in System Settings."],
  },
  {
    id: "windows",
    name: "Windows",
    requirement: "64-bit Windows on x64",
    short: "x64",
    assets: [{ suffix: "windows-x64-setup.exe", label: "Installer (.exe)" }],
    installAnchor: "windows-x64",
    notes: ["Not signed: SmartScreen shows a warning."],
  },
  {
    id: "linux",
    name: "Linux",
    requirement: "x86_64 with an X11 session",
    short: "x86_64, X11",
    assets: [
      { suffix: "linux-x86_64.AppImage", label: "AppImage" },
      { suffix: "linux-x86_64.tar.gz", label: "Archive (.tar.gz)" },
    ],
    installAnchor: "linux-x86_64-x11",
    notes: ["Wayland isn't supported."],
  },
];
