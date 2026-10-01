from pathlib import Path
import json,re,math
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parent
T=json.loads((ROOT/'source/tokens.json').read_text());S=json.loads((ROOT/'source/screens.json').read_text())
def kebab(s):return re.sub(r'([A-Z])',lambda m:'-'+m[1].lower(),s)
(ROOT/'source/tokens.css').write_text('/* Generated from source/tokens.json. Do not hand edit. */\n'+'\n'.join(sel+'{'+''.join('--'+kebab(k)+':'+v+';' for k,v in T['color'][mode].items())+'--motion-curve:'+T['motion']['curve']+';}' for mode,sel in [('light',':root'),('dark','[data-theme="dark"]')])+'\n')
(ROOT/'source/data.mjs').write_text('export const screens = '+json.dumps(S,ensure_ascii=False,indent=2)+';\n')
f=ImageFont.truetype(str(ROOT/'assets/OpenSans-Semibold.ttf'),14)
for mobile in [False,True]:
 mode='mobile' if mobile else'desktop';cols=6 if mobile else 3;tw=210 if mobile else 440;th=620 if mobile else 360;gap=20;im=Image.new('RGB',(cols*(tw+gap)+gap,math.ceil(len(S)/cols)*(th+42)+gap),'#F0F3F9');d=ImageDraw.Draw(im)
 for i,s in enumerate(S):
  p=ROOT/'previews'/f'{s["id"]}-{mode}.png'
  if not p.exists():continue
  a=Image.open(p).convert('RGB');a.thumbnail((tw,th));x=gap+(i%cols)*(tw+gap);y=gap+(i//cols)*(th+42);im.paste(a,(x,y));d.text((x,y+th+6),s['id'],fill='#1C2638',font=f)
 im.quantize(colors=256,method=Image.Quantize.MEDIANCUT).save(ROOT/'previews'/f'{mode}-contact-sheet.png',optimize=True)
print('Tokens, fixture data and contact sheets regenerated')
