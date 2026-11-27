// Snip mock-up data: theme palettes (from crates/kx-theme/tokens/themes.toml), ink colours, designs and steps.
'use strict';

// Palettes copied from selection-chips.html, which copied them from themes.toml.
const T = {
  native:{name:'Native', rKey:4, rWin:8, g:4, fs:13, fsMod:11, fw:400, ar:1.12, latin:'"Segoe UI Variable Text","Segoe UI",sans-serif', arabic:'"Segoe UI","Noto Sans Arabic",sans-serif',
    light:{plate:'#E9EBEB',key:'#FFFFFF',keyMod:'#EBEBEB',keyAct:'#F7F7F7',enter:'#005FB8',enterInk:'#FFFFFF',danger:'#EBEBEB',dangerInk:'#1B1B1B',legend:'#1B1B1B',legendMod:'#1B1B1B',legend2:'#5F5F5F',legendAct:'#1B1B1B',ring:'#005FB8',badgeBg:'#005FB8',badgeInk:'#FFFFFF',popBg:'#F9F9F9',popInk:'#1B1B1B',popMuted:'#5F5F5F',popLine:'#00000017',popHover:'#0000000B',popSel:'#DCE9F7',chipBg:'#0000000E',keyShadow:'inset 0 -1px 0 #00000024, 0 0 0 1px #0000000B',winShadow:'0 12px 32px #00000038, 0 0 0 1px #00000014'},
    dark:{plate:'#2C2F33',key:'#3B3E43',keyMod:'#34373C',keyAct:'#373A3F',enter:'#7CB7F0',enterInk:'#10202E',danger:'#34373C',dangerInk:'#E3E5E8',legend:'#E3E5E8',legendMod:'#E3E5E8',legend2:'#BABEC3',legendAct:'#E3E5E8',ring:'#7CB7F0',badgeBg:'#7CB7F0',badgeInk:'#10202E',popBg:'#33363B',popInk:'#E3E5E8',popMuted:'#A9AEB5',popLine:'#FFFFFF17',popHover:'#FFFFFF0E',popSel:'#2E4760',chipBg:'#FFFFFF12',keyShadow:'inset 0 -1px 0 #00000047, 0 0 0 1px #FFFFFF0A',winShadow:'0 14px 34px #00000066, 0 0 0 1px #FFFFFF0F'}},
  et66:{name:'ET66', rKey:9, rWin:14, g:5, fs:13, fsMod:10.5, fw:500, ar:1.14, latin:'"IBM Plex Sans",sans-serif', arabic:'"IBM Plex Sans Arabic","Noto Sans Arabic",sans-serif',
    light:{plate:'#E4E1DA',key:'#F8F6F1',keyMod:'#CDBBA7',keyAct:'#F8F6F1',enter:'#F2C400',enterInk:'#1D1B18',danger:'#CDBBA7',dangerInk:'#1D1B18',legend:'#1D1B18',legendMod:'#1D1B18',legend2:'#5D5850',legendAct:'#1D1B18',ring:'#947700',badgeBg:'#1D1B18',badgeInk:'#F2C400',popBg:'#FBFAF7',popInk:'#1D1B18',popMuted:'#6B655C',popLine:'#00000017',popHover:'#0000000A',popSel:'#F3E7C0',chipBg:'#F8F6F1',keyShadow:'0 1px 0 #3C322847, inset 0 1px 0 #FFFFFFBF',winShadow:'0 14px 34px #281E1440, 0 0 0 1px #0000000F'},
    dark:{plate:'#2A2826',key:'#3A3734',keyMod:'#6B5645',keyAct:'#3A3734',enter:'#E6BE3A',enterInk:'#2A2418',danger:'#6B5645',dangerInk:'#F0ECE4',legend:'#EDE8DF',legendMod:'#F3EEE6',legend2:'#C6C0B9',legendAct:'#EDE8DF',ring:'#E6BE3A',badgeBg:'#2A2418',badgeInk:'#E6BE3A',popBg:'#33302D',popInk:'#EDE8DF',popMuted:'#B3ACA2',popLine:'#FFFFFF14',popHover:'#FFFFFF0D',popSel:'#4A4130',chipBg:'#3A3734',keyShadow:'0 1px 0 #00000059, inset 0 1px 0 #FFFFFF0F',winShadow:'0 14px 36px #00000066, 0 0 0 1px #FFFFFF0D'}},
  dolch:{name:'Dolch', rKey:5, rWin:10, g:3, fs:13, fsMod:10.5, fw:500, ar:1.1, latin:'Rubik,sans-serif', arabic:'Rubik,"Noto Sans Arabic",sans-serif',
    light:{plate:'#B9BBBE',key:'#E6E7E8',keyMod:'#5F6161',keyAct:'#5F6161',enter:'#00797D',enterInk:'#FFFFFF',danger:'#B83262',dangerInk:'#F7F2EA',legend:'#393B3B',legendMod:'#F7F2EA',legend2:'#575759',legendAct:'#F7F2EA',ring:'#00696D',badgeBg:'#393B3B',badgeInk:'#7FE6E9',popBg:'#E6E7E8',popInk:'#393B3B',popMuted:'#5F6161',popLine:'#0000001F',popHover:'#0000000D',popSel:'#CBEDEE',chipBg:'#D4D6D8',keyShadow:'0 1px 1px #00000040',winShadow:'0 14px 34px #00000047, 0 0 0 1px #00000014',keySk:'#C5C7CA',modSk:'#4A4C4C',enterSk:'#006D70',dangerSk:'#952749'},
    dark:{plate:'#2A2C2E',key:'#4A4D4F',keyMod:'#35383B',keyAct:'#35383B',enter:'#137378',enterInk:'#FFFFFF',danger:'#A03D5A',dangerInk:'#F4F1EA',legend:'#EDEAE4',legendMod:'#EDEAE4',legend2:'#E2E4E4',legendAct:'#EDEAE4',ring:'#3CC0C4',badgeBg:'#168388',badgeInk:'#FFFFFF',popBg:'#34373A',popInk:'#EDEAE4',popMuted:'#B0B3B6',popLine:'#FFFFFF14',popHover:'#FFFFFF0D',popSel:'#1D4A4C',chipBg:'#3E4144',keyShadow:'0 1px 1px #00000059',winShadow:'0 14px 36px #0000006B, 0 0 0 1px #FFFFFF0D',keySk:'#3A3C3E',modSk:'#2B2D30',enterSk:'#106568',dangerSk:'#86314B'}}
};

// Drawing inks, close to Snipping Tool's pens; in the product they become theme tokens.
const INK = [['red','#E81123'],['yellow','#FFD400'],['green','#10893E'],['blue','#0063B1'],['black','#1B1B1B'],['white','#FFFFFF']];
// The recording edge and dot, and the frozen screen's dim and edge (common.toml snip_dim, snip_edge, snip_tag).
const REC = '#E81123', DIM = '#0000006B', EDGE = '#FFFFFF', TAG = '#00000099';

// The stage is one 1280 x 760 screen with a 40 px taskbar.
const W = 1280, H = 760, TASK = 40;
// The scene's windows and the parts each step picks.
const BOX = {
  web:{x:56, y:36, w:780, h:486}, notes:{x:872, y:96, w:350, h:300},
  region:{x:96, y:150, w:520, h:300}, video:{x:96, y:130, w:620, h:340}
};

const BARS = {
  tool:{label:'A Snipping Tool', say:'Bar A follows Windows Snipping Tool: a Photo and Video switch, then the shape.'},
  keys:{label:'B Keycaps', say:'Bar B is keycaps on the keyboard’s plate, like the strip: every way is one click, photo or video.'},
  dock:{label:'C Bottom dock', say:'Bar C follows the macOS Screenshot bar: photo ways on the left, video ways on the right, at the bottom.'}
};
const EDITORS = {top:'Tools on top', side:'Tools at the side'};
const KBS = {bubble:'Bubble', hidden:'Hidden', shown:'Shown'};
const STEPS = [
  ['bar','1 Capture bar'],['region','2 Region taken'],['window','3 Window'],['full','4 Full screen'],
  ['vready','5 Video area'],['count','6 Countdown'],['rec','7 Recording'],['vdone','8 Video done'],
  ['edit','9 Editor'],['vedit','10 Video editor'],['ask','11 Close asks'],['panel','12 Snip panel']
];

// A Lucide icon by its file name (kebab case), drawn from the UMD build; win-logo is our own drawing.
function ic(name, px) {
  const size = px ? ' width="' + px + '" height="' + px + '"' : '';
  if (name === 'win-logo') return '<svg class="i"' + size + ' viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3.5 3.5h8v8h-8zM12.5 3.5h8v8h-8zM3.5 12.5h8v8h-8zM12.5 12.5h8v8h-8z"/></svg>';
  const pascal = name.split('-').map(p => p.charAt(0).toUpperCase() + p.slice(1)).join('');
  const LU = window.lucide || {};
  const node = LU[pascal] || (LU.icons && LU.icons[pascal]);
  if (!node) return '<span class="i i-fb">' + name.charAt(0) + '</span>';
  const draw = n => n.map(([t, a, c]) => '<' + t + ' ' + Object.entries(a).map(([k, v]) => k + '="' + v + '"').join(' ') + '>' + (c ? draw(c) : '') + '</' + t + '>').join('');
  return '<svg class="i"' + size + ' viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' + draw(node) + '</svg>';
}
