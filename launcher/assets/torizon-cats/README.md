# Torizon cat mascots

Original graphics generated with the built-in OpenAI ImageGen tool for this project.
Prompt: a 4×2 sprite sheet of SFW Boykisser-style fluffy cartoon cats, with four
white-cat waving/blinking frames and four lavender-cat smiling/winking frames,
consistent scale, dark outlines, pink blush and no text or watermark.

`sprites.png` is the original generation. `tools/torizon_mascots.py` slices the
two rows and encodes `wave.gif` and `wink.gif` (four frames, infinite loop).
The PNG variants show a still frame while the game runs, so launcher decorations
do not keep animating in the background during gameplay.
