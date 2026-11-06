# Torizon animated mascots

Generated with the built-in ImageGen tool, then encoded as transparent GIFs by
`python tools/torizon_mascots.py`. No external character artwork is bundled.

`sprites.png` is a 4 by 4 RGBA atlas. The first eight cells make `wave.gif` and
the last eight make `wink.gif`. Each GIF is 256 by 256, eight frames, infinite loop,
with palette index 255 reserved for transparency and disposal 2. PNG stills keep RGBA.
The launcher draws the characters without a background panel. Animations pause during gameplay.

## Final image prompt

Use the previous sheet as character/style reference. Create one transparent RGBA
sprite atlas with exactly four columns and four rows, sixteen equal square cells.
No white backdrop, checkerboard, grid lines, cards, shadows or text. Keep white fur
opaque and the empty space transparent. First eight cells: the same white fluffy
Boykisser-inspired cartoon cat waist-up, anchored at center-bottom, waving a paw:
neutral, half lift, paw by face, outward swing, inward swing with closed-eye smile,
outward swing, half lower, neutral. Use clearly visible paw movement, blinking and
ear twitches, with consistent body scale. Last eight cells: the same pale lavender
cat with mint inner ears, cheerful side-to-side bob and wink: left tilt, center,
right tilt, center, happy eyes shut, right tilt, center, left tilt. Thick charcoal
outlines, opaque flat colors, pink blush, friendly nonsexual meme-cat style.
Contain every character within its cell with transparent margins, consistent scale
and fixed baseline. This is a production animation atlas, not a drawing of a grid.
