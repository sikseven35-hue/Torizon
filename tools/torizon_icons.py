"""Render Torizon's code-defined T mark into Windows and runtime icons (Pillow)."""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SVG = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
<title>Torizon</title><rect width="16" height="16" rx="3.5" fill="#141124"/>
<path d="M3 3h10v2H9v8H7V5H3z" fill="#56ded1"/>
</svg>\n'''

def render(size):
    scale = 4
    image = Image.new('RGBA', (size * scale, size * scale))
    draw = ImageDraw.Draw(image)
    unit = size * scale / 16
    draw.rounded_rectangle((0, 0, size * scale - 1, size * scale - 1),
                           radius=3.5 * unit, fill='#141124')
    draw.polygon([(x * unit, y * unit) for x, y in
                  [(3, 3), (13, 3), (13, 5), (9, 5), (9, 13), (7, 13), (7, 5), (3, 5)]],
                 fill='#56ded1')
    return image.resize((size, size), Image.Resampling.LANCZOS)

for path in ['docs/media/brand/icon.svg', 'docs/media/brand/icon-macos.svg',
             'packaging/icons/hicolor/scalable/apps/horizon.svg']:
    (ROOT / path).write_text(SVG, encoding='utf-8', newline='\n')
for path in (ROOT / 'packaging/icons/hicolor').glob('*x*/apps/horizon.png'):
    render(int(path.parents[1].name.split('x')[0])).save(path)
render(64).save(ROOT / 'crates/horizon-theme/icons/horizon-64.png')
render(256).save(ROOT / 'packaging/icons/horizon.ico', sizes=[(s, s) for s in [16, 24, 32, 48, 64, 128, 256]])
print('Torizon T icons rendered.')
