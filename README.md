# Hornet

Graphsim's **F/A-18 Hornet 2.0** (1995, classic Mac OS) on modern macOS,
through [Systemless](https://github.com/benletchford/systemless): a Rust
reimplementation of the Mac Toolbox that runs the original app with no ROM, no
System install and no machine emulator.

**Status: the 68K build flies with two small patches to Systemless.** Tested
headlessly on Linux with Graphsim's free 2.0 demo: title screen, briefing,
cockpit, engine start, takeoff roll and external view all render. Not tested
yet: the full retail game (it needs your copy) and a windowed session on a Mac.

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
(raw, DiskCopy 4.2, or Apple-partitioned CD images). A plain copy of the app
from a modern disk loses its resource fork, so use one of those forms.

Modern macOS cannot mount the original HFS CD, but it can still read the raw
disc. This is untested here:

```sh
diskutil list                          # find the CD drive, e.g. /dev/disk4
diskutil unmountDisk /dev/disk4
sudo dd if=/dev/rdisk4 of=hornet.iso bs=2048
tools/run-hornet.sh hornet.iso
```

If the disc holds an installer rather than the game, Systemless has installer
handling, but that path is untested for Hornet.

## Controls

These come from the demo's briefing screen. The pointer position is the
stick, so centre the mouse before you throttle up.

| Input | Action |
| --- | --- |
| Mouse forward / back | Dive / climb |
| Mouse left / right | Roll |
| Mouse button | Fire weapon |
| Delete | Start engines, then stage afterburner |
| `+` / `-` | Increase / decrease thrust |
| `G` | Gear up / down |
| `F` | Flaps |
| `R` | Radar air / ground |
| `[` | Air-to-air weapon select |
| `\` | Select radar target |
| `1` `2` `3` | Cockpit / look-down / external view |
| Esc | Stop flight |

The demo says joysticks are supported only in the full version. Joystick
support under Systemless is untested.

## The patches

Both patches apply to Systemless commit `38666a8` (0.96.0). The library test
suite passes with them applied.

**`0001-open-hfs-basenames-containing-slash.patch`** is the one that matters.
Without it, clicking *Fly Demo* shows "Sorry, I can't find the file
"F/A-18 Data"". The game opens its data file with
`PBOpen("F/A-18 Data 2.0 (demo)")`. On HFS only `:` separates path components,
so the `/` is part of the filename. Systemless stores such names with the slash
escaped (U+F02F), and `HOpen` already handles that. `find_vfs_file`, which
plain `PBOpen` uses, skips its basename fallback whenever the name contains a
`/`, so the file is never found. The patch lets the HFS-form basename fallback
run on its own, and adds a unit test. The retail data file is named
"F/A-18 Data", so the full game needs this patch too. The Systemless catalogue
recorded the demo reaching the cockpit on 0.51.0, so this is probably a
regression.

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
  implemented. 16 of its 343 imports are missing:
  `LMGetMainDevice`, `LMGetROM85`, `LMGetFCBSPtr`, `LMGetEventQueue`,
  `LMSetSEvtEnb`, `LMGetTEScrpHandle`, `LMSetTEScrpHandle`, `BitShift`,
  `DeltaPoint`, `AngleFromSlope`, `SystemEvent`, `RecoverHandleSys`,
  `DrvrInstall`, `PConfirmName`, `PrJobMerge` and `LNextCell`. Most are
  one-line low-memory accessors. `tools/pef_imports.py` lists a PEF's imports.
  The 68K build runs under Systemless's JIT, and `run-hornet.sh` passes
  `--prefer-68k`.
- `play/fly-demo.json` is the headless scenario. It waits for the cockpit's
  sky pixel and checks the instrument panel, so it fails if the data-file bug
  comes back.

## Not done yet

- The full retail game. Supply your copy, then run `run-hornet.sh` on it.
- Windowed play on macOS. Every run so far was headless, in a Linux container.
- Getting airborne in the scripted run. Pitch responds (angle of attack 2.7°),
  but the scripted mouse never cleared the runway. A person with a real mouse
  will.
- The PowerPC build, which needs the 16 imports above.
- Sending the patches upstream to Systemless.

## References

- [Systemless](https://github.com/benletchford/systemless) and its
  [Hornet 2.0 catalogue entry](https://github.com/benletchford/systemless/blob/main/www/catalogue/fa-18-hornet-20.md)
- [Systemless issue #2713](https://github.com/benletchford/systemless/issues/2713):
  the earlier Hornet fix (File Manager FCB refnums)
- [Hornet 2.0 demo](https://classicmacdemos.com/fa-18-hornet-20): the
  publisher's bundled docs permit free distribution
