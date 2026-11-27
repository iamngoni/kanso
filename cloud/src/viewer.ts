// The page a published note is read on: `GET /p/<share id>#<key>`
// (docs/ACCOUNTS.md › Published notes).
//
// Static HTML with one inline style and one inline script, both pinned in the
// Content-Security-Policy by their SHA-256, and nothing loaded from anywhere
// else. The script reads the key from location.hash (the fragment never
// reaches the server), fetches the ciphertext from /v2/public/<id>, opens it
// with WebCrypto's AES-256-GCM, and shows the note in an iframe sandboxed
// without allow-scripts and without allow-same-origin.
//
// A srcdoc frame inherits this page's policy, so the note's own styling has
// to fit it: the script gathers the note's <style> blocks and style=""
// attributes into one stylesheet the frame loads as a data: URL (`style-src
// data:`), and its images are data: URLs already (`img-src data:`). Both are
// taken out as text before the page is parsed: a DOMParser document inherits
// the policy too, so the browser would refuse to parse them there. Nothing in
// the note can run, load from the network, or reach this page.
//
// A published notebook (docs/ACCOUNTS.md › Published notebooks) is one share
// too: its plaintext is JSON, `{"kanso":"site",…}`, with the notebook's index
// and each note's page. `#<key>` shows the index and `#<key>/<slug>` a page.
// Links between its pages (`kanso-page:<slug>`) become links to this page at
// the other page's fragment, opened in the top window: the frame may
// navigate it only when the reader clicks (allow-top-navigation-by-user-
// activation), and a link to the same page with another fragment never
// leaves it, so the key stays in the fragment. Links to other places still
// open in a new tab. Images come back from the bundle's files as data:
// URLs; other attachments are offered to download in the bar above the page.

import { DEFAULT_TITLE, PREVIEW_DESCRIPTION, PREVIEW_IMAGE_PATH } from "./rerout";

/** The page's only stylesheet. Calm: Kanso's paper and graphite, in both schemes. */
export const VIEWER_STYLE = `:root{color-scheme:light dark;--paper:#fbfaf7;--ink:#2b2a27;--muted:#76736c;--line:#e6e2da;--card:#ffffff}
@media (prefers-color-scheme:dark){:root{--paper:#1c1b19;--ink:#e9e6df;--muted:#9a968d;--line:#34322e;--card:#232220}}
*{box-sizing:border-box}
html,body{margin:0;height:100%;background:var(--paper);color:var(--ink);font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif}
#note{position:fixed;inset:0;width:100%;height:100%;border:0;display:none;background:var(--paper)}
#note.shown{display:block}
#bar{display:none;position:fixed;top:0;left:0;right:0;height:44px;align-items:center;gap:14px;padding:0 16px;border-bottom:1px solid var(--line);background:var(--card);overflow:hidden}
body.site #bar{display:flex}
body.site #note{top:44px;height:calc(100% - 44px)}
#bar a{color:var(--ink);font-weight:600;text-decoration:none;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
#bar .files{margin-left:auto;display:flex;gap:8px;overflow:hidden}
#bar button{font:inherit;font-size:13px;color:var(--ink);background:none;border:1px solid var(--line);border-radius:6px;padding:3px 9px;cursor:pointer;white-space:nowrap}
.card a{color:var(--ink)}
#status{min-height:100%;display:flex;align-items:center;justify-content:center;padding:24px}
#status.hidden{display:none}
.card{max-width:420px;padding:28px 30px;border:1px solid var(--line);border-radius:12px;background:var(--card)}
.card h1{margin:0 0 8px;font-size:17px;font-weight:600;letter-spacing:-.01em}
.card p{margin:0;color:var(--muted)}
.quiet{color:var(--muted)}`;

/** A published notebook as its bundle carries it (kanso_sync::site::SiteBundle). */
export interface SiteBundle {
	kanso: "site";
	v: 1;
	title: string;
	index: string;
	pages: Record<string, { title: string; html: string; downloads?: number[] }>;
	files: { name: string; type: string; data: string }[];
}

// The helpers below are part of the viewer's script, as their source: the
// tests call the same functions the page runs. They use only `var` and
// anonymous callbacks, so their source is the same however it is bundled.

/** `#<key>` or `#<key>/<slug>`: the key as written and the page's slug, "" for the index. */
export function siteRoute(hash: string): { key: string; slug: string } {
	var text = hash.charAt(0) === "#" ? hash.slice(1) : hash;
	var cut = text.indexOf("/");
	return cut < 0 ? { key: text, slug: "" } : { key: text.slice(0, cut), slug: text.slice(cut + 1) };
}

/** Where a `kanso-page:<slug>` link goes on the viewer at `page` with `key`: that page's fragment, or null for a page the site doesn't have. */
export function siteHref(href: string, pages: Record<string, unknown>, page: string, key: string): string | null {
	if (href.indexOf("kanso-page:") !== 0) return null;
	var slug = href.slice(11);
	if (slug !== "" && !(/^[a-z0-9-]{1,64}$/.test(slug) && Object.prototype.hasOwnProperty.call(pages, slug))) return null;
	return page + "#" + key + (slug ? "/" + slug : "");
}

/** A page's images back from the bundle: `src="kanso-file:<n>"` as the file's data: URL when it is an image, else empty. */
export function siteImages(html: string, files: { type: string; data: string }[]): string {
	return html.replace(/src="kanso-file:(\d{1,6})"/g, function (_whole: string, n: string) {
		var file = files[Number(n)];
		var ok = !!file && /^image\/[A-Za-z0-9.+-]{1,60}$/.test(file.type) && /^[A-Za-z0-9+\/]*={0,2}$/.test(file.data);
		return ok ? 'src="data:' + file.type + ";base64," + file.data + '"' : 'src=""';
	});
}

/** The bundle, when the opened text is a published notebook's; null for a note's page. */
export function siteBundle(text: string): SiteBundle | null {
	if (text.charAt(0) !== "{") return null;
	var site;
	try {
		site = JSON.parse(text);
	} catch (e) {
		return null;
	}
	var ok =
		!!site && site.kanso === "site" && site.v === 1 && typeof site.title === "string" && typeof site.index === "string" &&
		!!site.pages && typeof site.pages === "object" && !Array.isArray(site.pages) && Array.isArray(site.files);
	return ok ? site : null;
}

/** The page's only script. Every step that can fail ends in a calm message. */
export const VIEWER_SCRIPT = `(function(){
"use strict";
${siteRoute.toString()}
${siteHref.toString()}
${siteImages.toString()}
${siteBundle.toString()}
var status=document.getElementById("status"),frame=document.getElementById("note"),bar=document.getElementById("bar");
var site=null,note=null;
function say(title,detail,home){
status.textContent="";
var card=document.createElement("div");card.className="card";
var h=document.createElement("h1");h.textContent=title;card.appendChild(h);
var p=document.createElement("p");p.textContent=detail;card.appendChild(p);
if(home){var a=document.createElement("a");a.href=home;a.textContent="Go to the notebook\u2019s first page";var q=document.createElement("p");q.appendChild(a);card.appendChild(q);}
status.appendChild(card);status.className="";frame.className="";document.title=title;
}
function b64url(text){
if(!/^[A-Za-z0-9_-]*$/.test(text)||text.length%4===1)return null;
var s=text.replace(/-/g,"+").replace(/_/g,"/");while(s.length%4)s+="=";
try{var bin=atob(s);}catch(e){return null;}
var out=new Uint8Array(bin.length);for(var i=0;i<bin.length;i++)out[i]=bin.charCodeAt(i);return out;
}
var id=(location.pathname.match(/^\\/p\\/([A-Za-z0-9_-]{22})$/)||[])[1];
var route=siteRoute(location.hash);
var key=b64url(route.key);
var here=location.origin+location.pathname;
if(!id){say("This link isn\\u2019t complete","Check that the whole link was copied.");return;}
if(!key||key.length!==32){say("This link is missing its key","The part after # unlocks the note. Ask whoever shared it for the whole link.");return;}
function clean(html){
var css="";
html=html.replace(/<style\\b[^>]*>([\\s\\S]*?)<\\/style>/gi,function(_,text){css+=text.replace(/<!\\[CDATA\\[|\\]\\]>/g,"")+"\\n";return "";});
html=html.replace(/(<[^<>]*?\\s)style\\s*=/gi,"$1data-kanso-css=");
var doc=new DOMParser().parseFromString(html,"text/html");
doc.querySelectorAll("script,style,template,iframe,frame,object,embed,base,link,meta,form,noscript").forEach(function(el){el.remove();});
var n=0;
doc.querySelectorAll("*").forEach(function(el){
for(var i=el.attributes.length-1;i>=0;i--){var name=el.attributes[i].name;if(/^on/i.test(name)||name==="style"||name==="target")el.removeAttribute(name);}
if(el.hasAttribute("data-kanso-css")){
var rules="";
(el.getAttribute("data-kanso-css")||"").split(/;(?![^(]*\\))/).forEach(function(decl){
var at=decl.indexOf(":");if(at<1)return;
var p=decl.slice(0,at).trim().toLowerCase(),v=decl.slice(at+1).trim().replace(/!\\s*important$/i,"").trim();
if(/^-?[a-z][a-z0-9-]*$/.test(p)&&v&&!/[{}<>\\\\]/.test(v))rules+=p+":"+v+" !important;";
});
el.removeAttribute("data-kanso-css");
if(rules){n++;el.setAttribute("data-kanso-style",String(n));css+="[data-kanso-style=\\""+n+"\\"]{"+rules+"}\\n";}
}
});
doc.querySelectorAll("a[href],area[href]").forEach(function(a){
var href=a.getAttribute("href")||"";
if(site&&href.indexOf("kanso-page:")===0){
var to=siteHref(href,site.pages,here,route.key);
if(to){a.setAttribute("href",to);a.setAttribute("target","_top");}else a.removeAttribute("href");
return;
}
if(href.charAt(0)==="#")return;
if(/^(https?:|mailto:)/i.test(href)){a.setAttribute("target","_blank");a.setAttribute("rel","noopener noreferrer");}
else a.removeAttribute("href");
});
var head=doc.head||doc.documentElement.insertBefore(doc.createElement("head"),doc.body);
var ref=doc.createElement("meta");ref.setAttribute("name","referrer");ref.setAttribute("content","no-referrer");head.insertBefore(ref,head.firstChild);
css+="@media screen and (min-width:48em){body{max-width:max(40rem,75vw)}}figure svg{width:100%!important;max-width:100%!important;height:auto!important}\\n";
var link=doc.createElement("link");link.setAttribute("rel","stylesheet");link.setAttribute("href","data:text/css;charset=utf-8,"+encodeURIComponent(css));head.appendChild(link);
var title=(doc.title||"").trim();
return {html:"<!doctype html>"+doc.documentElement.outerHTML,title:title};
}
function shown(page,fallback){
document.title=page.title||fallback;
frame.srcdoc=page.html;frame.className="shown";status.className="hidden";
}
function download(file){
var bin;try{bin=atob(file.data);}catch(e){return;}
var bytes=new Uint8Array(bin.length);for(var i=0;i<bin.length;i++)bytes[i]=bin.charCodeAt(i);
var url=URL.createObjectURL(new Blob([bytes],{type:"application/octet-stream"}));
var a=document.createElement("a");a.href=url;a.download=String(file.name||"file");document.body.appendChild(a);a.click();a.remove();
setTimeout(function(){URL.revokeObjectURL(url);},10000);
}
function showSite(){
var slug=route.slug,page=slug===""?{title:site.title,html:site.index,downloads:[]}:(/^[a-z0-9-]{1,64}$/.test(slug)&&Object.prototype.hasOwnProperty.call(site.pages,slug)?site.pages[slug]:null);
bar.textContent="";
var home=document.createElement("a");home.href=here+"#"+route.key;home.textContent=site.title||"Notebook";bar.appendChild(home);
if(!page){say("This page isn\\u2019t in the notebook","It may have been left out when the notebook was published again.",here+"#"+route.key);return;}
var files=document.createElement("span");files.className="files";
(Array.isArray(page.downloads)?page.downloads:[]).forEach(function(ix){
var file=site.files[ix];if(!file)return;
var b=document.createElement("button");b.type="button";b.textContent=String(file.name||"file");b.title="Download "+b.textContent;
b.addEventListener("click",function(){download(file);});files.appendChild(b);
});
bar.appendChild(files);
shown(clean(siteImages(String(page.html),site.files)),site.title||"A notebook");
}
window.addEventListener("hashchange",function(){
var next=siteRoute(location.hash);
if(next.key!==route.key){location.reload();return;}
route=next;
if(site)showSite();
});
fetch("/v2/public/"+id,{cache:"no-store",credentials:"omit",referrerPolicy:"no-referrer"}).then(function(r){
if(r.status===404)throw "gone";
if(r.status===410)throw "expired";
if(r.status===429)throw "busy";
if(!r.ok)throw "server";
return r.arrayBuffer();
}).then(function(buf){
var bytes=new Uint8Array(buf);
if(bytes.length<29||bytes[0]!==1)throw "key";
var aad=new TextEncoder().encode("kanso-share\\u0001"+id);
return crypto.subtle.importKey("raw",key,{name:"AES-GCM"},false,["decrypt"]).then(function(k){
return crypto.subtle.decrypt({name:"AES-GCM",iv:bytes.slice(1,13),additionalData:aad,tagLength:128},k,bytes.slice(13));
}).catch(function(){throw "key";});
}).then(function(plain){
var text=new TextDecoder().decode(plain);
site=siteBundle(text);
if(site){document.body.className="site";showSite();return;}
note=clean(text);
shown(note,"A note");
}).catch(function(why){
if(why==="gone")say("This note isn\\u2019t published any more","It may have been unpublished, or the link may be mistyped.");
else if(why==="expired")say("This link has expired","Whoever shared it chose for it to stop working after a while.");
else if(why==="busy")say("Too many requests","Wait a minute, then reload the page.");
else if(why==="key")say("This key doesn\\u2019t open the note","Check that the whole link was copied, including the part after #.");
else say("Couldn\\u2019t load the note","Check your connection, then reload the page.");
});
})();`;

const escapeAttr = (text: string): string => text.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/**
 * The page: the style and script above, exactly as hashed. Its preview tags
 * are the same for every share (the page never reads D1): "A note", one line
 * about Kanso and Kanso's image on `origin`.
 */
export const viewerHtml = (origin: string): string => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow">
<meta name="referrer" content="no-referrer">
<title>A note</title>
<meta property="og:site_name" content="Kanso">
<meta property="og:title" content="${DEFAULT_TITLE}">
<meta property="og:description" content="${PREVIEW_DESCRIPTION}">
<meta property="og:image" content="${escapeAttr(origin + PREVIEW_IMAGE_PATH)}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta name="twitter:card" content="summary_large_image">
<style>${VIEWER_STYLE}</style>
</head>
<body>
<nav id="bar"></nav>
<main id="status"><p class="quiet">Opening the note…</p></main>
<iframe id="note" title="The note" sandbox="allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation" referrerpolicy="no-referrer"></iframe>
<script>${VIEWER_SCRIPT}</script>
</body>
</html>
`;

async function cspHash(text: string): Promise<string> {
	const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)));
	let binary = "";
	for (const b of digest) binary += String.fromCharCode(b);
	return `'sha256-${btoa(binary)}'`;
}

let policy: Promise<string> | null = null;

/**
 * The viewer's Content-Security-Policy: its own script and style by hash,
 * fetches to this origin only, and for the note in the frame (which inherits
 * the policy) data: images, fonts and the gathered stylesheet. No frame may
 * hold the page, and nothing posts anywhere.
 */
export function viewerPolicy(): Promise<string> {
	policy ??= (async () =>
		[
			"default-src 'none'",
			`script-src ${await cspHash(VIEWER_SCRIPT)}`,
			`style-src ${await cspHash(VIEWER_STYLE)} data:`,
			"img-src data:",
			"font-src data:",
			"connect-src 'self'",
			"frame-src 'none'",
			"base-uri 'none'",
			"form-action 'none'",
			"frame-ancestors 'none'",
		].join("; "))();
	return policy;
}

/** Headers every public answer carries: no referrer, no sniffing, no framing, not indexed. */
export const PUBLIC_HEADERS: Record<string, string> = {
	"referrer-policy": "no-referrer",
	"x-content-type-options": "nosniff",
	"x-frame-options": "DENY",
	"x-robots-tag": "noindex, nofollow",
	"cross-origin-opener-policy": "same-origin",
	"cross-origin-resource-policy": "same-origin",
};

/** `GET /p/<id>`: the page, the same for every share. */
export async function viewerPage(origin: string): Promise<Response> {
	return new Response(viewerHtml(origin), {
		headers: {
			"content-type": "text/html; charset=utf-8",
			"content-security-policy": await viewerPolicy(),
			"cache-control": "public, max-age=300",
			"permissions-policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
			...PUBLIC_HEADERS,
		},
	});
}
