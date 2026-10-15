# Hornet

Graphsim's **F/A-18 Hornet 2.0** (1995, classic Mac OS) on modern macOS,
through [Systemless](https://github.com/benletchford/systemless): a Rust
reimplementation of the Mac Toolbox that runs the original app with no ROM, no
System install and no machine emulator.

**Status: the full retail game reaches the cockpit with two small patches to
Systemless.** Tested headlessly on Linux with the 68K build.

- **Retail 2.0.1:** launches straight from the installer disc. Registration,
  pilot dossier, briefing, menus, the Load Armament screen, the carrier-deck
  cockpit, engine start and the deck roll all work.
- **Free 2.0 demo:** the title screen, briefing, cockpit, engine start,
  takeoff roll and external view all work.

A windowed session on a Mac is not tested yet.

## What works today

```sh
tools/setup-systemless.sh               # clone Systemless @38666a8, apply patches/, build
tools/smoke-demo.sh                     # fetch the free 2.0 demo, fly it headlessly
tools/run-hornet.sh path/to/Hornet.sit  # play it in a window
```

Needs Rust from [rustup.rs](https://rustup.rs). On Linux, also install
`libasound2-dev`. The release build takes several minutes because of its LTO
settings; `PROFILE=ci-test` builds faster and runs a little slower.
Everything the scripts download or build goes in `out/`, which git ignores.

## Using your copy of the full game

Systemless reads StuffIt (`.sit`), MacBinary (`.bin`) and HFS disk images
(raw, DiskCopy 4.2, or Apple-partitioned CD images such as Toast). A plain
copy of the app from a modern disk loses its resource fork, so use one of
those forms.

The installer disc works as-is, with no install step:

```sh
tools/run-hornet.sh "F-A-18 Hornet 2.0.toast"       # or the .sit holding it
```

This was tested with a Toast image of the retail installer disc (volume
"F/A-18 Hornet 2.0", March 1996), both bare and inside a StuffIt archive. The
disc holds an Installer VISE 3 app. Systemless unpacks it on load and finds the
`F/A-18 Hornet Folder`, which contains `F/A-18 Hornet 2.0.1` and its 11 MB
`F/A-18 Data 2.0`. It launches the game rather than the installer or the
bundled Acrobat Reader.

The first launch asks for the serial number from the Product Registration
card in the back of the Pilot Flight Manual. The disc also carries the full
manual as `Hornet 2.0 Manual.pdf`.

Modern macOS cannot mount the original HFS CD, but it can still read the raw
disc. This is untested here:

```sh
diskutil list                          # find the CD drive, e.g. /dev/disk4
diskutil unmountDisk /dev/disk4
sudo dd if=/dev/rdisk4 of=hornet.iso bs=2048
tools/run-hornet.sh hornet.iso
```

## Flying

The pointer position is the stick, so centre the mouse before you throttle
up. Pick a mission from the **Mission** menu: *Training…* for the six
training missions (the first starts on a runway), or *Fly* for the current
Tour of Duty mission. *Fly* opens the Load Armament screen; click **Fly**
there to reach the cockpit.

| Input | Action |
| --- | --- |
| Mouse forward / back | Dive / climb |
| Mouse left / right | Roll |
| Mouse button | Fire weapon |
| Delete | Throttle to 100%, then each press adds an afterburner stage |
| `+` / `-` | Increase / decrease thrust |
| Space bar | Wheel brake on the ground, speed brake in the air |
| `G` | Gear up / down |
| `F` | Flaps |
| `R` | Radar air / ground |
| `[` | Air-to-air weapon select |
| `\` | Select radar target |
| `D` | Damage report |
| `1` `2` `3` | Cockpit / look-down / external view |
| Esc | Stop flight |

**Carrier launch** (manual, page 25):

1. Taxi onto the base of a catapult.
2. Hold the Space bar until the jet stops completely; the brake locks.
3. Raise thrust to 100%. The nose dips as the catapult tightens.
4. Press and release the Space bar to launch, then pull back on the mouse.

The Kuwait mission starts on the carrier deck, but not on a catapult.

Joysticks are untested under Systemless.

## The patches

Both patches apply to Systemless commit `38666a8` (0.96.0). The library test
suite passes with them applied.

**`0001-open-hfs-basenames-containing-slash.patch`** is the one that matters.
Without it the game says "Sorry, I can't find the file "F/A-18 Data"". On HFS
only `:` separates path components, so the `/` in "F/A-18 Data 2.0" is part
of the filename. Systemless stores such names with the slash escaped
(U+F02F), and `HOpen` already handles that. Its two name lookups do not:
`find_vfs_file` (data forks, used by `PBOpen`) and `find_vfs_rsrc_file`
(resource forks, used by `OpenResFile`). Both skip their basename fallback
whenever the name contains a `/`, so the file is never found.

The demo trips the data-fork case. The retail game opens its data file's
resource fork first and trips the resource-fork case. The patch lets both
lookups match a colon-free HFS name as a basename, and adds a unit test for
each. The Systemless catalogue recorded the demo reaching the cockpit on
0.51.0, so this is probably a regression.

**`0002-ppc-alias-tesetscraplength.patch`** is PowerPC only.
`TESetScrapLength` is the same call as `LMSetTEScrpLength`, which Systemless
already implements; the alias was missing from its import table. On its own it
does not make the PowerPC build run (see below).

## Things to know

- **Choose the app explicitly.** The demo archive also contains the
  publisher's docs reader, which is a PowerPC app. Systemless prefers PowerPC,
  so a plain `systemless demo.sit` opens the docs. `run-hornet.sh` sets
  `SYSTEMLESS_LOAD_EXECUTABLE=Hornet` to pick the game.
- **Run the 68K build.** Hornet 2.0 is a fat binary: its data fork is a PEF
  container (`Joy!peffpwpc`). Systemless picks the PowerPC build by default,
  and that build stops at once on InterfaceLib imports that Systemless has not
  implemented. 16 of the demo's 343 imports are missing:
  `LMGetMainDevice`, `LMGetROM85`, `LMGetFCBSPtr`, `LMGetEventQueue`,
  `LMSetSEvtEnb`, `LMGetTEScrpHandle`, `LMSetTEScrpHandle`, `BitShift`,
  `DeltaPoint`, `AngleFromSlope`, `SystemEvent`, `RecoverHandleSys`,
  `DrvrInstall`, `PConfirmName`, `PrJobMerge` and `LNextCell`. Most are
  one-line low-memory accessors. `tools/pef_imports.py` lists a PEF's imports.
  The 68K build runs under Systemless's JIT, and `run-hornet.sh` passes
  `--prefer-68k`.
- **No network play.** The game probes for the AppleTalk driver (`.MPP`),
  which Systemless does not provide, so *Network Host* and *Network Join* stay
  greyed out.
- **Saves.** The game keeps its registration in
  `Preferences:F/A-18 Preferences`. Systemless persists each game's System
  Folder preferences between windowed launches (`--reset-preferences` clears
  them), so registering once should be enough. That is not verified yet.
  Headless play-script runs always start fresh.
- `play/fly-demo.json` is the headless scenario for the demo. It waits for
  the cockpit's sky pixel and checks the instrument panel, so it fails if the
  data-file bug comes back.

## Known issues

- On the retail Briefing page, the mission paragraph is drawn on top of
  another text block, so it is unreadable. The Time, Conditions and
  Destination lines below it are fine.

## Not done yet

- Windowed play on macOS. Every run so far was headless, in a Linux container.
- Getting airborne in a scripted run. Pitch and brakes respond, but the
  scripted mouse never cleared the runway and the carrier runs never caught a
  catapult. A person flying with a real mouse should manage both.
- The PowerPC build, which needs the 16 imports above.
- Sending the patches upstream to Systemless.

## References

- [Systemless](https://github.com/benletchford/systemless) and its
  [Hornet 2.0 catalogue entry](https://github.com/benletchford/systemless/blob/main/www/catalogue/fa-18-hornet-20.md)
- [Systemless issue #2713](https://github.com/benletchford/systemless/issues/2713):
  the earlier Hornet fix (File Manager FCB refnums)
- [Hornet 2.0 demo](https://classicmacdemos.com/fa-18-hornet-20): the
  publisher's bundled docs permit free distribution
