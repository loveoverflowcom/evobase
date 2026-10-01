"""Original layered SVG design frames, rasterized by Inkscape. Not browser screenshots."""
from pathlib import Path
from html import escape
import json,subprocess,os,sys,math,hashlib
from PIL import ImageFont,Image,ImageDraw
ROOT=Path(__file__).resolve().parent
T=json.loads((ROOT/'source/tokens.json').read_text());SCREENS=json.loads((ROOT/'source/screens.json').read_text());BY={s['id']:s for s in SCREENS}
FONT=str(ROOT/'assets/OpenSans-Regular.ttf');BOLD=str(ROOT/'assets/OpenSans-Semibold.ttf')
CFG=Path('/tmp')/('evobase-fontconfig-'+hashlib.sha256(str(ROOT).encode()).hexdigest()[:8]+'.xml')
CFG.write_text('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>'+escape(str(ROOT/'assets'))+'</dir><cachedir>/tmp/evobase-font-cache</cachedir></fontconfig>')
NAV_BUILDER=['Bảng dữ liệu','Cấu trúc & Quan hệ','Quy tắc & Quyền','Chế độ xem','Hành động & Quy trình','Kết nối','Xuất bản']
NAV_RUNTIME=['Đơn hàng','Công việc','Tự động hóa','Hộp thư','Đồng bộ']
NAV_WORK=['Ứng dụng','Thành phần','Trạng thái','Cài đặt']
class Canvas:
 def __init__(self,s,m=False,dark=False):
  self.s=s;self.m=m;self.dark=dark;self.w=390 if m else 1440;self.h=1190 if m else 1000;self.c=T['color']['dark' if dark else 'light'];self.p=[];self.maxy=0
  self.rect(0,0,self.w,self.h,self.c['surfaceContainerLow']);self.text(16 if m else 28,19,'EVOBASE · '+s['id']+' · STATIC DESIGN / SAMPLE DATA',8,'onSurfaceVariant',600)
 def col(self,c):return self.c.get(c,c)
 def rect(self,x,y,w,h,fill='surface',r=16,stroke=None,sw=1):
  self.p.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{self.col(fill)}"'+(f' stroke="{self.col(stroke)}" stroke-width="{sw}"' if stroke else '')+'/>');self.maxy=max(self.maxy,y+h)
 def line(self,x1,y1,x2,y2,c='outlineVariant',sw=1):self.p.append(f'<path d="M{x1} {y1}L{x2} {y2}" stroke="{self.col(c)}" stroke-width="{sw}" fill="none"/>')
 def text(self,x,y,t,size=14,c='onSurface',weight=400,anchor='start',mono=False):
  self.p.append(f'<text x="{x}" y="{y}" font-family="{"DejaVu Sans Mono" if mono else "Open Sans"}" font-size="{size}" font-weight="{weight}" fill="{self.col(c)}" text-anchor="{anchor}">{escape(str(t))}</text>');self.maxy=max(self.maxy,y+4)
 def para(self,x,y,t,w,size=13,c='onSurfaceVariant',weight=400,lh=1.6):
  f=ImageFont.truetype(BOLD if weight>400 else FONT,size);lines=[];l=''
  for wd in str(t).split():
   test=(l+' '+wd).strip()
   if f.getlength(test)>w and l:lines.append(l);l=wd
   else:l=test
  if l:lines.append(l)
  for i,l in enumerate(lines):self.text(x,y+i*size*lh,l,size,c,weight)
  return len(lines)*size*lh
 def pill(self,x,y,t,kind='tonal',w=None):
  f=ImageFont.truetype(BOLD,10);w=w or f.getlength(t)+24;self.rect(x,y,w,26,'warningContainer' if kind=='warn' else'errorContainer' if kind=='error' else'primaryContainer' if kind=='primary' else'surfaceContainer',13);self.text(x+12,y+18,t,10,'onWarningContainer' if kind=='warn' else'onErrorContainer' if kind=='error' else'onPrimaryContainer' if kind=='primary' else'onSurfaceVariant',600);return w
 def button(self,x,y,w,t,filled=True,h=48):
  self.rect(x,y,w,h,'primary' if filled else'primaryContainer',h/2);self.text(x+w/2,y+h/2+4,t,11 if self.m else 12,'onPrimary' if filled else'onPrimaryContainer',600,'middle')
 def icon(self,x,y,name,size=22,c='onSurfaceVariant'):
  paths={'grid':'M3 3H19V19H3ZM3 9H19M9 3V19','relations':'M4 5H8V9H4ZM14 13H18V17H14ZM8 7H16V13','policy':'M11 2L19 5V11C19 16 11 20 11 20S3 16 3 11V5ZM7 10L10 13L16 7','view':'M2 6H20V18H2ZM2 10H20M8 10V18','flow':'M4 2V7H9V12H16V18M1 2H7M13 18H19','connect':'M7 4V9M15 4V9M5 9H17V13C17 17 5 17 5 13ZM11 17V21','release':'M4 4H18V18H4ZM8 11L11 8L14 11M11 8V16','home':'M3 10L11 3L19 10V19H3Z','task':'M4 3H18V19H4ZM7 7L9 9L13 5M7 14H15','chat':'M3 4H19V16H10L5 20V16H3Z','sync':'M4 8A8 8 0 0 1 18 6L20 3V9H14M18 14A8 8 0 0 1 4 16L2 19V13H8','settings':'M11 4A7 7 0 1 1 10.9 4M11 8A3 3 0 1 1 10.9 8'}
  d=paths.get(name,paths['grid']);self.p.append(f'<g transform="translate({x} {y}) scale({size/22})"><path d="{d}" fill="none" stroke="{self.col(c)}" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/></g>')
 def shell(self):
  s=self.s;m=self.m;self.rect(0,28,self.w,68 if not m else 60,'surface',0)
  raw=(ROOT/'assets/evobase-mark.svg').read_text().split('>',1)[1].rsplit('</svg>',1)[0];self.p.append(f'<g transform="translate({16 if m else 28} 44) scale(.48)">{raw}</g>');self.text(57 if m else 70,67,'EvoBase',18,'onSurface',600)
  if not m:
   self.pill(245,47,s['surface'],'primary',103);self.text(370,67,'Không gian   /   Bán hàng',13,'onSurfaceVariant');self.rect(977,42,310,44,'surfaceContainerLow',22);self.text(996,70,'Tìm bảng, record, hành động…',12,'onSurfaceVariant');self.rect(1352,42,44,44,'tertiaryContainer',18);self.text(1374,70,'LA',12,'onTertiaryContainer',600,'middle')
   self.rect(20,112,216,848,'surface',24);self.text(40,146,'KHÔNG GIAN',10,'onSurfaceVariant',600);self.text(40,173,'Bán hàng',18,'onSurface',600)
   nav=NAV_BUILDER if s['surface']=='Builder' else NAV_RUNTIME if s['surface']=='Runtime' else ['Tenant & Lưu trữ'] if s['surface']=='Platform' else NAV_WORK
   names=['grid','relations','policy','view','flow','connect','release'] if s['surface']=='Builder' else ['home','task','flow','chat','sync'] if s['surface']=='Runtime' else ['grid','view','task','settings']
   for i,n in enumerate(nav):
    y=201+i*57;sel=n==s['section'];self.rect(30,y,196,49,'primaryContainer' if sel else'surface',18);self.icon(44,y+13,names[i%len(names)],20,'onPrimaryContainer' if sel else'onSurfaceVariant');self.text(75,y+30,n,10.5,'onPrimaryContainer' if sel else'onSurfaceVariant',600 if sel else 400)
   self.line(40,776,214,776);self.pill(40,799,'BẢN NHÁP 12' if s['surface']=='Builder' else'PHIÊN BẢN 1.2' if s['surface']=='Runtime' else'DESIGN KIT','primary',174);self.para(40,849,'Builder và Runtime có quyền và vòng đời riêng.',169,11);self.text(40,931,'?  Trợ giúp',12,'onSurfaceVariant')
  else:
   
   if s['id'] in ['runtime-detail','runtime-form','approval','run-detail','inbox']:
    self.rect(8,37,192,51,'surface',12);self.rect(10,40,48,48,'surfaceContainerLow',24);self.text(34,72,'←',22,'primary',600,'middle');self.text(65,71,'Quay lại',13,'onSurface',600)
   self.pill(209,47,s['surface'],'primary',83);self.rect(326,43,44,44,'tertiaryContainer',18);self.text(348,71,'LA',12,'onTertiaryContainer',600,'middle')
 def header(self):
  x=16 if self.m else 264;w=358 if self.m else 1132;y=121 if self.m else 134
  self.text(x,y,self.s['kicker'],9 if self.m else 10,'onSurfaceVariant',600)
  yy=y+38;dh=self.para(x,yy,self.s['title'],w if self.m else w-300,27 if self.m else 36,'onSurface',600,1.28); yy+=dh+8
  dh=self.para(x,yy,self.s['description'],w if self.m else w-310,12 if self.m else 13);yy+=dh+9
  self.pill(x,yy,self.s['status'],'warn' if 'Unknown' in self.s['status'] or'gate' in self.s['status'] or'cần' in self.s['status'] or'Offline' in self.s['status'] else'primary');yy+=44
  if not self.m:self.button(1151,153,245,self.s['action'])
  if self.m and self.s['surface']=='Builder':
   self.rect(x,yy,w,70,'secondaryContainer',18);self.text(x+16,yy+25,'BUILDER · BẢN TÓM TẮT',10,'onSecondaryContainer',600);self.para(x+16,yy+46,'Chỉnh cấu trúc trên màn hình rộng. Mobile ưu tiên Runtime.',w-32,11,'onSecondaryContainer');yy+=86
  return x,yy,w
 def panel(self,x,y,w,h,title=None,fill='surface'):
  self.rect(x,y,w,h,fill,24)
  if title:self.text(x+22,y+36,title,17,'onSurface',600)
 def cards(self,x,y,w,secs,columns=2,compact=False):
  cols=1 if self.m else columns;gap=16;cw=(w-(cols-1)*gap)/cols;heights=[]
  for i,s in enumerate(secs):
   xx=x+(i%cols)*(cw+gap);yy=y+(i//cols)*(160 if compact else 186);hh=144 if compact else 170;self.panel(xx,yy,cw,hh,fill='surface' if i%3 else'primaryContainer');self.text(xx+20,yy+32,f'{i+1:02}',10,'primary',600)
   n=self.para(xx+20,yy+62,s[0],cw-40,17 if self.m else 18,'onSurface',600,1.3);n2=self.para(xx+20,yy+62+n+12,s[1],cw-40,12,'onSurfaceVariant');self.para(xx+20,yy+hh-24,s[2],cw-40,10,'onSurfaceVariant')
  return y+math.ceil(len(secs)/cols)*(160 if compact else 186)
 def fields(self,x,y,w,fields,columns=1):
  cols=1 if self.m else columns;cw=(w-(cols-1)*16)/cols
  for i,(label,value) in enumerate(fields):
   xx=x+(i%cols)*(cw+16);yy=y+(i//cols)*94;self.text(xx,yy+12,label,11,'onSurfaceVariant',600);self.rect(xx,yy+25,cw,54,'surfaceContainerLow',12);self.para(xx+15,yy+48,value,cw-30,12,'onSurface')
  return y+math.ceil(len(fields)/cols)*94
 def inspector(self,x,y,w,items,h=None):
  h=h or 72+len(items)*64;self.panel(x,y,w,h,'Chi tiết & ràng buộc','surfaceContainer')
  for i,(a,b) in enumerate(items):
   self.text(x+20,y+76+i*64,a,10,'onSurfaceVariant',600);self.para(x+20,y+101+i*64,b,w-40,12,'onSurface',600)
  return y+h
 def footer(self,y):
  x=16 if self.m else 264;w=358 if self.m else 1132;y+=24
  if self.m:
   self.h=max(920,y+150);self.rect(0,self.h-132,390,132,'surface',0);a='Tiếp tục trên màn hình rộng' if self.s['surface']=='Builder' else self.s['action'];self.button(16,self.h-118,358,a);self.line(0,self.h-57,390,self.h-57)
   
   if self.s['surface']=='Runtime':
    tabs=['Đơn hàng','Công việc','Hộp thư','Khác'];icons=['home','task','chat','settings'];selected=1 if self.s['section']=='Công việc' else 2 if self.s['section']=='Hộp thư' else 0 if self.s['section']=='Đơn hàng' else 3
   elif self.s['surface']=='Workspace':
    tabs=['Ứng dụng','Thành phần','Trạng thái','Cài đặt'];icons=['grid','view','task','settings'];selected={'Ứng dụng':0,'Thành phần':1,'Trạng thái':2,'Cài đặt':3}.get(self.s['section'],0)
   else:
    tabs=['Tổng quan','Bảng','Cấu trúc','Khác'];icons=['home','grid','relations','settings'];selected=1 if self.s['section']=='Bảng dữ liệu' else 2 if self.s['section']=='Cấu trúc & Quan hệ' else 3
   for i,n in enumerate(tabs):xx=49+i*97;self.icon(xx-11,self.h-45,icons[i],19,'primary' if i==selected else'onSurfaceVariant');self.text(xx,self.h-10,n,8,'primary' if i==selected else'onSurfaceVariant',600,'middle')
  else:self.h=max(1000,y+40);self.text(264,self.h-22,'DESIGN REFERENCE ONLY · Static SVG raster · Không phải browser screenshot hoặc production evidence',9,'onSurfaceVariant')
 def write(self,path):
  self.p[0]=f'<rect width="{self.w}" height="{self.h}" fill="{self.c["surfaceContainerLow"]}"/>'
  path.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" viewBox="0 0 {self.w} {self.h}"><title>{escape(self.s["title"])}</title><desc>Original static design reference, sample data; not runtime screenshot.</desc>'+''.join(self.p)+'</svg>')
def grid(c,x,y,w):
 if c.m:
  yy=c.cards(x,y,w,[[r[0]+' · '+r[1],r[2]+' / '+r[3],'Phụ trách: '+r[4]] for r in c.s['rows']],compact=True);return c.inspector(x,yy+4,w,c.s['inspector'])
 mw=w-304;ix=x+mw+20;c.panel(x,y,mw,535)
 tabs=['Orders','Customers','OrderLines','Products'];tx=x+16
 for i,t in enumerate(tabs):tx+=c.pill(tx,y+17,t,'primary' if i==0 else'tonal')+8
 c.rect(x+16,y+62,mw-32,46,'surfaceContainerLow',14);c.text(x+31,y+91,'⌕  Tìm đơn hàng',12,'onSurfaceVariant');c.pill(x+mw-262,y+72,'Tất cả trạng thái');c.text(x+mw-105,y+91,'+ Bộ lọc',11,'primary',600)
 widths=[120,176,122,150,mw-32-568];xx=x+16
 for j,hd in enumerate(c.s['columns']):c.text(xx+10,y+139,hd,10,'onSurfaceVariant',600);xx+=widths[j]
 for i,row in enumerate(c.s['rows']):
  yy=y+151+i*55;c.rect(x+16,yy,mw-32,54,'primaryContainer' if i==1 else'surface',8);xx=x+16
  for j,v in enumerate(row):
   if j==1:c.pill(xx+8,yy+14,v,'primary')
   elif j==2:c.pill(xx+8,yy+14,v,'tonal')
   else:c.text(xx+10,yy+33,v,11,'onSurface',600 if j==0 else 400)
   xx+=widths[j]
  c.line(x+16,yy+54,x+mw-16,yy+54)
  if i==1:c.rect(x+16+120,yy+2,176,50,'none',8,'primary',3)
 c.text(x+26,y+465,'5 / 148 bản ghi mẫu · page 1',10,'onSurfaceVariant');c.button(x+mw-156,y+449,125,'Trang tiếp',False,38)
 c.rect(x+16,y+495,mw-32,29,'surfaceContainerLow',8);c.text(x+29,y+515,'↵ Edit     ⇥ Next cell     Esc Cancel     Ctrl+Z Undo     IME-safe',9,'onSurfaceVariant')
 c.inspector(ix,y,284,c.s['inspector'],535)
 return y+535

def relations(c,x,y,w):
 if c.m:return c.cards(x,y,w,c.s['sections'])
 mw=w-304;c.panel(x,y,mw,542);selfnodes=[]
 pos=[(x+28,y+63),(x+mw-308,y+63),(x+28,y+304),(x+mw-308,y+304)];order=[0,1,3,2]
 for pi,i in enumerate(order):
  xx,yy=pos[pi];s=c.s['sections'][i];c.rect(xx,yy,260,177,'primaryContainer' if i==1 else'surfaceContainerLow',20);c.text(xx+20,yy+33,s[0],18,'onSurface',600);c.para(xx+20,yy+64,s[1],220,11,'onSurfaceVariant');c.line(xx+20,yy+104,xx+240,yy+104);c.para(xx+20,yy+133,s[2],220,10,'primary',600)
 c.line(x+288,y+152,x+mw-308,y+152,'primary',2);c.pill(x+mw/2-28,y+137,'1 : N','primary',64);c.line(x+mw-178,y+240,x+mw-178,y+304,'primary',2);c.pill(x+mw-208,y+259,'1 : N','primary',64);c.line(x+288,y+393,x+mw-308,y+393,'tertiary',2);c.pill(x+mw/2-28,y+378,'1 : N','primary',64)
 c.text(x+28,y+514,'Ref identity → canonical value · edges/indexes là projection',11,'onSurfaceVariant');c.inspector(x+mw+20,y,284,c.s['inspector'],542);return y+542

def flow(c,x,y,w):
 steps=c.s['steps'];mw=w if c.m else w-304;yy=y;c.panel(x,y,mw,len(steps)*86+74,'Các bước có kiểu')
 for i,(a,b) in enumerate(steps):
  sy=y+59+i*86;c.rect(x+20,sy,mw-40,70,'primaryContainer' if i==0 else'surfaceContainerLow',18);c.rect(x+33,sy+14,42,42,'primary' if i==0 else'tertiaryContainer',14);c.text(x+54,sy+40,str(i+1),16,'onPrimary' if i==0 else'onTertiaryContainer',600,'middle');c.text(x+88,sy+26,a,12,'onSurface',600);c.para(x+88,sy+50,b,mw-116,11,'onSurfaceVariant')
  if i<len(steps)-1:c.line(x+54,sy+71,x+54,sy+85,'outline',2)
 yy=y+len(steps)*86+74
 if 'inspector' in c.s:
  if c.m:yy=c.inspector(x,yy+16,w,c.s['inspector'])
  else:c.inspector(x+mw+20,y,284,c.s['inspector'],max(394,yy-y))
 return yy

def form(c,x,y,w):
 mw=w if c.m else (w-320 if 'inspector' in c.s else 720);fields=c.s.get('fields',[]);h=84+math.ceil(len(fields)/(1 if c.m else 2))*94;c.panel(x,y,mw,h,'Thông tin có kiểu');yy=c.fields(x+20,y+60,mw-40,fields,1 if c.m else 2)+24
 sec=c.s.get('sections',[])
 if c.m:return c.cards(x,yy+16,w,sec,compact=True) if sec else yy
 sx=x+mw+20;sw=w-mw-20
 if 'inspector' in c.s:c.inspector(sx,y,sw,c.s['inspector'])
 elif sec:c.cards(sx,y,sw,sec,columns=1)
 return max(yy,y+len(sec)*186)

def review(c,x,y,w):
 mw=w if c.m or' ins' in c.s else w-304 if 'inspector' in c.s else w;yy=c.cards(x,y,mw,c.s['sections'],columns=1 if 'inspector' in c.s else 2)
 if 'inspector' in c.s:
  if c.m:yy=c.inspector(x,yy+8,w,c.s['inspector'])
  else:c.inspector(x+mw+20,y,284,c.s['inspector'],yy-y-16)
 return yy

def hub(c,x,y,w):
 h=270 if c.m else 240;c.rect(x,y,w,h,'primaryContainer',32);c.text(x+24,y+34,'TỪ BẢNG → ỨNG DỤNG',10,'onPrimaryContainer',600);c.para(x+24,y+83,'Dữ liệu rõ nghĩa.\nCông việc liền mạch.',w-48 if c.m else 560,29 if c.m else 38,'onPrimaryContainer',600,1.25);c.para(x+24,y+h-29,'Schema / relations / rules / release',w-48,12,'onPrimaryContainer')
 if not c.m:
  raw=(ROOT/'assets/relation-orbit.svg').read_text().split('>',1)[1].rsplit('</svg>',1)[0];c.p.append(f'<g transform="translate({x+w-494} {y+1}) scale(.9)">{raw}</g>')
 return c.cards(x,y+h+20,w,c.s['sections'],columns=3)

def runtime(c,x,y,w):
 h=101 if c.m else 116;c.rect(x,y,w,h,'tertiaryContainer',28);c.text(x+20,y+29,'CHỜ BẠN XỬ LÝ',10,'onTertiaryContainer',600);c.text(x+20,y+75,'3',36,'onTertiaryContainer',600);c.text(x+66,y+72,'yêu cầu cần duyệt',17,'onTertiaryContainer',600);c.text(x+w-24,y+72,'→',22,'onTertiaryContainer',600,'end')
 yy=y+h+28;c.text(x,yy,'Cần quyết định',18,'onSurface',600);yy+=18
 for r in c.s['tasks']:
  c.rect(x,yy,w,82 if c.m else 76,'surface',16);c.text(x+18,yy+24,r[0]+' · '+r[1],12,'onSurface',600);c.text(x+18,yy+47,r[2],14,'onSurface',600);c.text(x+18,yy+68,r[3]+' · '+r[4],10,'onSurfaceVariant')
  if c.m:c.pill(x+w-102,yy+18,'Cần duyệt','primary',84)
  else:c.button(x+w-157,yy+14,138,'Xem yêu cầu',False)
  yy+=90 if c.m else 84
 yy+=22;c.text(x,yy,'Đơn hàng gần đây',18,'onSurface',600);yy+=18
 for r in c.s['recent']:
  c.rect(x,yy,w,78 if c.m else 74,'surface',16);c.text(x+18,yy+25,r[0]+' · '+r[1],12,'onSurface',600);c.text(x+18,yy+49,r[2],13,'onSurface',600);c.text(x+w-18,yy+49,r[5],10,'primary',600,'end');c.text(x+18,yy+67,r[3]+' · '+r[4],9,'onSurfaceVariant');yy+=86 if c.m else 82
 return yy

def detail(c,x,y,w):
 mw=w if c.m else 720;h=98+len(c.s['fields'])*58;c.panel(x,y,mw,h,'Tóm tắt record')
 for i,(a,b) in enumerate(c.s['fields']):sy=y+68+i*58;c.text(x+20,sy,a,10,'onSurfaceVariant',600);c.para(x+20 if c.m else x+245,sy+22 if c.m else sy,b,mw-40 if c.m else mw-266,12,'onSurface',600)
 c.pill(x+20,y+h-42,'System metadata · chỉ đọc','primary')
 secs=c.s['sections'];yy=y+h+16
 if c.m:yy=c.cards(x,yy,w,secs,compact=True)
 else:c.cards(x+740,y,w-740,secs,columns=1);yy=max(yy,y+len(secs)*186)
 if c.s['id']=='approval':c.button(x+20,yy+4,mw-40,'Từ chối · cần lý do',False);yy+=64
 return yy

def chat(c,x,y,w):
 mw=w if c.m else w-304;c.panel(x,y,mw,518,'Minh An · ORD-1048');yy=y+70
 for i,s in enumerate(c.s['sections'][:2]):
  bx=x+20 if i==0 else x+56;bw=mw-76;c.rect(bx,yy,bw,133,'surfaceContainerLow' if i==0 else'primaryContainer',20);c.text(bx+16,yy+29,s[0],12,'onSurface',600);c.para(bx+16,yy+58,s[1],bw-32,12,'onSurface');c.text(bx+16,yy+112,s[2],10,'onSurfaceVariant');yy+=156
 c.rect(x+20,y+402,mw-40,82,'surfaceContainerLow',14);c.text(x+35,y+428,'Soạn tin nhắn…',12,'onSurfaceVariant');c.text(x+35,y+467,'Draft · chưa có external dispatch',10,'onSurfaceVariant');yy=y+518
 if c.m:yy=c.inspector(x,yy+16,w,c.s['inspector'])
 else:c.inspector(x+mw+20,y,284,c.s['inspector'],518)
 return yy

def components(c,x,y,w):
 yy=c.cards(x,y,w,c.s['sections'],columns=2);h=258 if c.m else 172;c.panel(x,yy,w,h,'Action / Ref / Focus')
 c.button(x+20,yy+60,min(190,w-40),'Hành động chính');c.button(x+20 if c.m else x+226,yy+118 if c.m else yy+60,min(190,w-40),'Hành động phụ',False)
 cx=x+20 if c.m else x+442;cy=yy+179 if c.m else yy+68;c.pill(cx,cy,'↗ Minh An','primary',142);c.rect(cx-4,cy-4,150,34,'none',18,'primary',3)
 return yy+h

def render(s,m=False,dark=False):
 c=Canvas(s,m,dark);c.shell();x,y,w=c.header();kind=s['kind'];fun={'grid':grid,'relations':relations,'flow':flow,'timeline':flow,'form':form,'editor':form,'policy':form,'review':review,'states':review,'ops':review,'connectors':review,'hub':hub,'runtime':runtime,'detail':detail,'chat':chat,'components':components}[kind];yy=fun(c,x,y,w);c.footer(yy)
 name=s['id']+'-'+('mobile' if m else'desktop')+('-dark' if dark else'');svg=ROOT/'frames'/f'{name}.svg';png=ROOT/'previews'/f'{name}.png';c.write(svg)
 env=os.environ.copy();env['FONTCONFIG_FILE']=str(CFG);env['XDG_CACHE_HOME']='/tmp/evobase-font-cache';subprocess.run(['inkscape',str(svg),'--export-type=png','--export-filename='+str(png)],check=True,capture_output=True,env=env)
 Image.open(png).convert('RGB').quantize(colors=256,method=Image.Quantize.MEDIANCUT).save(png,optimize=True)
 return {'screen':s['id'],'variant':'mobile' if m else'desktop','theme':'dark' if dark else'light','width':c.w,'height':c.h,'evidence':'static-svg-raster','path':str(png.relative_to(ROOT))}
if __name__=='__main__':
 fs=sys.argv[1:] or [s['id'] for s in SCREENS]
 for f in fs:
  for m in [False,True]:print(json.dumps(render(BY[f],m),ensure_ascii=False),flush=True)
