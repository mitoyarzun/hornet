"""Generate a play script that captures one frame every 2 ticks (30 fps)."""
import json, sys
acts = []
n = 0
def shoot(ticks):
    global n
    for _ in range(ticks // 2):
        acts.append({"type": "run", "ticks": 2})
        acts.append({"type": "screenshot", "name": f"f{n:05d}"})
        n += 1
def act(*a): acts.extend(a)
def click(v, h):
    act({"type": "mouse_down", "v": v, "h": h}); shoot(4)
    act({"type": "mouse_up", "v": v, "h": h})
def key(code, ch):
    act({"type": "key_down", "key": code, "ch": ch}); shoot(4)
    act({"type": "key_up", "key": code, "ch": ch})
def move(v, h): act({"type": "mouse_move", "v": v, "h": h})

act({"type": "run", "ticks": 120})
shoot(180)                      # title screen
click(513, 607); shoot(240)     # interactive demo briefing
click(301, 52)
act({"type": "run_until_pixel", "x": 150, "y": 100, "rgb": [184, 218, 236], "timeout_ticks": 900})
move(300, 400); shoot(90)       # cockpit
key(51, 8); shoot(200)          # engines to 100%
for _ in range(3): key(51, 8); shoot(10)   # afterburner stages
shoot(1000)                     # takeoff roll
move(440, 400); shoot(70)       # rotate
move(345, 400); shoot(300)      # climb
move(300, 400); shoot(120)
move(300, 470); shoot(40)       # bank right
move(300, 400); shoot(200)
key(20, 51); shoot(300)         # external view
move(300, 340); shoot(40)       # bank left
move(300, 400); shoot(200)
key(18, 49); shoot(120)         # back to cockpit
print(json.dumps({"version": 1, "clock": "frontend_ticks", "max_ticks": 20000, "actions": acts}))
print(n, file=sys.stderr)
