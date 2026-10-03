from pathlib import Path
import json,math,hashlib,re,zipfile,xml.etree.ElementTree as ET,subprocess
from PIL import Image
ROOT=Path(__file__).resolve().parent;T=json.loads((ROOT/'source/tokens.json').read_text());S=json.loads((ROOT/'source/screens.json').read_text());fails=[];checks=[]
def check(ok,label):
 checks.append({'check':label,'pass':bool(ok)})
 if not ok:fails.append(label)
check(len(S)==27 and len({s['id'] for s in S})==27,'27 unique screens')
check(len({s['pack'] for s in S})==12,'12 canonical packs')
check(set(T['color']['light'])==set(T['color']['dark']),'paired light/dark role names')
def lum(v):
 a=[int(v[i:i+2],16)/255 for i in [1,3,5]];a=[x/12.92 if x<=.04045 else((x+.055)/1.055)**2.4 for x in a];return sum(x*w for x,w in zip(a,[.2126,.7152,.0722]))
contrasts=[]
for mode,C in T['color'].items():
 for fg,bg in [('onPrimary','primary'),('onPrimaryContainer','primaryContainer'),('onSecondaryContainer','secondaryContainer'),('onTertiaryContainer','tertiaryContainer'),('onErrorContainer','errorContainer'),('onSurface','surface'),('onSurfaceVariant','surfaceContainerHighest'),('onWarningContainer','warningContainer'),('onSuccessContainer','successContainer')]:
  l1,l2=sorted([lum(C[fg]),lum(C[bg])]);ratio=(l2+.05)/(l1+.05);check(ratio>=4.5,f'{mode}:{fg}/{bg} text contrast');contrasts.append({'theme':mode,'foreground':fg,'background':bg,'ratio':round(ratio,2)})
for s in S:
 for mode in ['desktop','mobile']:
  png=ROOT/'previews'/f'{s["id"]}-{mode}.png';svg=ROOT/'frames'/f'{s["id"]}-{mode}.svg';check(png.exists() and svg.exists(),s['id']+':'+mode+':frame+PNG')
  rt=ET.parse(svg).getroot();sz=Image.open(png).size;check(abs(float(rt.attrib['width'])-sz[0])<1 and abs(float(rt.attrib['height'])-sz[1])<1,s['id']+':'+mode+':raster dimensions')
  check(all(float(r.attrib.get('width',0))>=0 and float(r.attrib.get('height',0))>=0 for r in rt.iter('{http://www.w3.org/2000/svg}rect')),s['id']+':'+mode+':no negative rects')
for f in ['app.mjs','data.mjs']:check(subprocess.run(['node','--check',str(ROOT/'source'/f)],capture_output=True).returncode==0,f+':Node syntax')
html=(ROOT/'source/index.html').read_text();css=(ROOT/'source/style.css').read_text();js=(ROOT/'source/app.mjs').read_text();check('aria-labelledby="dialog-title"' in html and 'role="status"' in html,'dialog/status semantic hooks');check('prefers-reduced-motion:reduce' in css and 'forced-colors:active' in css,'reduced motion + forced colors');check("byId.get('workspace')||screens[0]" in js,'scoped pack first-screen fallback');check('aria-current="page"' in js,'active surface dock/rail markers')
manifest=json.loads((ROOT/'packages/pack-index.json').read_text())
for p in manifest['packs']:
 z=ROOT/p['zip'];check(z.stat().st_size<=1_500_000,p['pack']+':bounded ZIP <1.5MB');check(hashlib.sha256(z.read_bytes()).hexdigest()==p['sha256'],p['pack']+':ZIP SHA256');
 with zipfile.ZipFile(z) as a:
  check(a.testzip() is None,p['pack']+':ZIP integrity');sc=json.loads(a.read('source/screens.json'));check([x['id'] for x in sc]==p['screens'] or set(x['id'] for x in sc)==set(p['screens']),p['pack']+':scoped screen data');check('assets/OpenSans-NOTICE.txt' in a.namelist() and 'assets/LICENSE-Apache-2.0.txt' in a.namelist(),p['pack']+':Apache font license + attribution')
textpaths=list((ROOT/'source').glob('*'))+list((ROOT/'specs').glob('*.md'))
secret=re.compile(r'AKIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|gh[pousr]_[A-Za-z0-9]{30,}')
check(not any(secret.search(p.read_text()) for p in textpaths if p.is_file()),'no recognized secret patterns in authored text')
result={'evidence':'static-source + static-svg-raster only','screen_count':len(S),'frame_count':len(list((ROOT/'frames').glob('*.svg'))),'passed':len(checks)-len(fails),'failed':fails,'contrasts':contrasts,'checks':checks,'not_run':['browser rendering/interaction','native CMP Android/iOS','Leptos mounted UI','Rust/storage/provider tests','full VI/EN localization']}
(ROOT/'qa/static-report.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['evidence','screen_count','frame_count','passed','failed','not_run']},ensure_ascii=False));raise SystemExit(1 if fails else 0)
