// Deliberately small DOM adapter. No filesystem, process, native modules or raw sockets.
(() => {
  'use strict';
  // Atomics.wait blocks in native code and cannot be stopped by JS interrupts.
  delete globalThis.Atomics; delete globalThis.SharedArrayBuffer;
  const MAX_NODES = 6000, MAX_TEXT = 2 * 1024 * 1024;
  let count = 0, serial = 0, document, writing = null, session = 0, navigation = null;
  const nodes = new Map(), timers = new Map();
  const parse = globalThis.__parse, resource = globalThis.__resource, resolve = globalThis.__resolve;
  delete globalThis.__parse; delete globalThis.__resource; delete globalThis.__resolve;
  const escape = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  const bounded = v => { const s = String(v ?? ''); if (s.length > MAX_TEXT) throw Error('DOM text limit'); return s; };
  class Event {
    constructor(type, options = {}) { this.type = type; this.bubbles = !!options.bubbles; this.cancelable = !!options.cancelable; this.defaultPrevented = false; this._stop = false; }
    preventDefault() { if (this.cancelable) this.defaultPrevented = true; }
    stopPropagation() { this._stop = true; }
  }
  class Target {
    constructor() { this._events = Object.create(null); }
    addEventListener(type, fn, options) { if (typeof fn === 'function') (this._events[type] ||= []).push({fn, once: !!options?.once}); }
    removeEventListener(type, fn) { this._events[type] = (this._events[type] || []).filter(e => e.fn !== fn); }
    dispatchEvent(event) {
      if (!event.target) event.target = this;
      event.currentTarget = this;
      for (const item of [...(this._events[event.type] || [])]) {
        item.fn.call(this, event);
        if (item.once) this.removeEventListener(event.type, item.fn);
      }
      let handler = this['on' + event.type];
      if (!handler && this.getAttribute?.('on' + event.type)) handler = new Function('event', this.getAttribute('on' + event.type));
      if (typeof handler === 'function' && handler.call(this, event) === false) event.preventDefault();
      if (event.bubbles && !event._stop) (this.parentNode || (this === document ? window : null))?.dispatchEvent(event);
      return !event.defaultPrevented;
    }
  }
  function descendants(node) { const out = []; for (const child of node.childNodes) { out.push(child); out.push(...descendants(child)); } return out; }
  function matches(node, selector) {
    if (node.nodeType !== 1) return false;
    return selector.split(',').some(part => {
      const chain = part.trim().split(/\s+/);
      let here = node;
      const simple = (n, s) => {
        if (!n || n.nodeType !== 1) return false;
        const tag = s.match(/^[a-zA-Z][\w-]*|^\*/)?.[0];
        if (tag && tag !== '*' && n.localName !== tag.toLowerCase()) return false;
        let rest = s.slice(tag?.length || 0);
        while (rest) {
          const token = rest.match(/^([#.])([\w-]+)|^\[([\w-]+)(?:=["']?([^\]"']*)["']?)?\]/);
          if (!token) return false;
          if (token[1] === '#' && n.id !== token[2]) return false;
          if (token[1] === '.' && !n.className.split(/\s+/).includes(token[2])) return false;
          if (token[3] && (!n.hasAttribute(token[3]) || (token[4] !== undefined && n.getAttribute(token[3]) !== token[4]))) return false;
          rest = rest.slice(token[0].length);
        }
        return true;
      };
      if (!simple(here, chain.pop())) return false;
      while (chain.length) { const parent = chain.pop(); do { here = here.parentNode; } while (here && !simple(here, parent)); if (!here) return false; }
      return true;
    });
  }
  class Node extends Target {
    constructor(tag = '', text = '') {
      super(); if (++count > MAX_NODES) throw Error('DOM node limit');
      this._id = ++serial; nodes.set(this._id, this); this.localName = tag.toLowerCase();
      this.nodeType = tag === '#document' ? 9 : tag ? 1 : 3;
      this._text = bounded(text); this._attrs = Object.create(null); this.childNodes = []; this.parentNode = null;
      this.style = {cssText: ''}; this.ownerDocument = document;
    }
    get nodeName() { return this.nodeType === 3 ? '#text' : this.localName.toUpperCase(); }
    get tagName() { return this.nodeName; }
    get children() { return this.childNodes.filter(n => n.nodeType === 1); }
    get firstChild() { return this.childNodes[0] || null; }
    get lastChild() { return this.childNodes.at(-1) || null; }
    get nextSibling() { return this.parentNode?.childNodes[this.parentNode.childNodes.indexOf(this) + 1] || null; }
    get parentElement() { return this.parentNode?.nodeType === 1 ? this.parentNode : null; }
    get isConnected() { let n = this; while (n.parentNode) n = n.parentNode; return n === document; }
    appendChild(child) { return this.insertBefore(child, null); }
    insertBefore(child, before) {
      if (!(child instanceof Node) || child === this || child.contains(this)) throw Error('Invalid DOM insertion');
      let depth = 0; for (let n = this; n; n = n.parentNode) if (++depth > 64) throw Error('DOM depth limit');
      if (depth + child._depth() > 64) throw Error('DOM depth limit');
      if (before !== null && before.parentNode !== this) throw Error('Reference node not found');
      child.remove(); const index = before ? this.childNodes.indexOf(before) : this.childNodes.length;
      this.childNodes.splice(index, 0, child); child.parentNode = this; return child;
    }
    _depth() { return 1 + Math.max(0, ...this.childNodes.map(n => n._depth())); }
    removeChild(child) { const i = this.childNodes.indexOf(child); if (i < 0) throw Error('Child not found'); this.childNodes.splice(i, 1); child.parentNode = null; return child; }
    replaceChild(child, old) { this.insertBefore(child, old); this.removeChild(old); return old; }
    remove() { this.parentNode?.removeChild(this); }
    append(...items) { items.forEach(n => this.appendChild(n instanceof Node ? n : document.createTextNode(n))); }
    prepend(...items) { const first = this.firstChild; items.forEach(n => this.insertBefore(n instanceof Node ? n : document.createTextNode(n), first)); }
    replaceChildren(...items) { [...this.childNodes].forEach(n => n.remove()); this.append(...items); }
    contains(n) { while (n) { if (n === this) return true; n = n.parentNode; } return false; }
    cloneNode(deep = false) { const n = new Node(this.localName, this._text); Object.assign(n._attrs, this._attrs); if (deep) this.childNodes.forEach(c => n.appendChild(c.cloneNode(true))); return n; }
    get textContent() { return this.nodeType === 3 ? this._text : this.childNodes.map(n => n.textContent).join(''); }
    set textContent(value) { if (this.nodeType === 3) this._text = bounded(value); else this.replaceChildren(document.createTextNode(value)); }
    get innerText() { return this.textContent; } set innerText(v) { this.textContent = v; }
    get nodeValue() { return this.nodeType === 3 ? this._text : null; } set nodeValue(v) { if (this.nodeType === 3) this._text = bounded(v); }
    get innerHTML() { return this.childNodes.map(n => serialize(n, false)).join(''); }
    set innerHTML(value) { this.replaceChildren(...fragment(bounded(value))); }
    get outerHTML() { return serialize(this, false); }
    insertAdjacentHTML(position, html) { const list = fragment(bounded(html)); if (position === 'beforeend') this.append(...list); else if (position === 'afterbegin') this.prepend(...list); else if (this.parentNode) { const before = position === 'beforebegin' ? this : this.nextSibling; list.forEach(n => this.parentNode.insertBefore(n, before)); } }
    getAttribute(name) { return Object.hasOwn(this._attrs, name) ? this._attrs[name] : null; }
    setAttribute(name, value) { if (!/^[\w:-]+$/.test(name) || name.startsWith('data-phone-')) return; this._attrs[name] = bounded(value); }
    removeAttribute(name) { delete this._attrs[name]; }
    hasAttribute(name) { return Object.hasOwn(this._attrs, name); }
    toggleAttribute(name, force) { const yes = force ?? !this.hasAttribute(name); if (yes) this.setAttribute(name, ''); else this.removeAttribute(name); return yes; }
    get id() { return this.getAttribute('id') || ''; } set id(v) { this.setAttribute('id', v); }
    get className() { return this.getAttribute('class') || ''; } set className(v) { this.setAttribute('class', v); }
    get classList() { const n = this; return {contains: s => n.className.split(/\s+/).includes(s), add: (...s) => n.className = [...new Set([...n.className.split(/\s+/), ...s])].join(' '), remove: (...s) => n.className = n.className.split(/\s+/).filter(x => !s.includes(x)).join(' '), toggle(s, force) { const yes = force ?? !this.contains(s); if (yes) this.add(s); else this.remove(s); return yes; }}; }
    get dataset() { const n = this; return new Proxy({}, {get: (_, k) => n.getAttribute('data-' + String(k).replace(/[A-Z]/g, c => '-' + c.toLowerCase())), set: (_, k, v) => { n.setAttribute('data-' + String(k).replace(/[A-Z]/g, c => '-' + c.toLowerCase()), v); return true; }}); }
    querySelectorAll(s) { const list = descendants(this).filter(n => matches(n, String(s))); list.item = i => list[i] || null; return list; }
    querySelector(s) { return this.querySelectorAll(s)[0] || null; }
    getElementsByTagName(s) { return this.querySelectorAll(s); }
    getElementsByClassName(s) { return this.querySelectorAll('.' + s.trim().split(/\s+/).join('.')); }
    matches(s) { return matches(this, s); } closest(s) { for (let n = this; n; n = n.parentNode) if (matches(n, s)) return n; return null; }
    get value() { if (this.localName === 'textarea') return this._value ?? this.textContent; if (this.localName === 'select') return this.options.find(n => n.selected)?.value || ''; return this._value ?? this.getAttribute('value') ?? (this.localName === 'option' ? this.textContent : ''); }
    set value(v) { this._value = bounded(v); if (this.localName === 'select') this.options.forEach(n => n.selected = n.value === this._value); }
    get checked() { return this._checked ?? this.hasAttribute('checked'); } set checked(v) { this._checked = !!v; }
    get selected() { return this._selected ?? this.hasAttribute('selected'); } set selected(v) { this._selected = !!v; }
    get options() { return this.querySelectorAll('option'); }
    get selectedIndex() { return this.options.findIndex(n => n.selected); } set selectedIndex(i) { this.options.forEach((n,j) => n.selected = i === j); }
    get form() { return this.closest('form'); }
    get elements() { return this.querySelectorAll('input,textarea,select,button'); }
    get href() { return resolve(document.URL, this.getAttribute('href') || ''); } set href(v) { this.setAttribute('href', v); }
    get src() { return resolve(document.URL, this.getAttribute('src') || ''); } set src(v) { this.setAttribute('src', v); }
    focus() { document.activeElement = this; } blur() { document.activeElement = null; }
    click() { activate(this); }
    submit() { submit(this, null); } requestSubmit(button) { if (this.dispatchEvent(new Event('submit', {bubbles:true,cancelable:true}))) submit(this, button); }
  }
  for (const name of ['name','type','placeholder','action','method']) Object.defineProperty(Node.prototype, name, {get() { return this.getAttribute(name) || ''; }, set(v) { this.setAttribute(name,v); }});
  for (const name of ['hidden','disabled','readOnly','multiple']) { const attr = name.toLowerCase(); Object.defineProperty(Node.prototype, name, {get() { return this.hasAttribute(attr); }, set(v) { this.toggleAttribute(attr,!!v); }}); }
  function build(data) { const n = new Node(data.tag || '', data.text || ''); for (const [k,v] of Object.entries(data.attrs || {})) n.setAttribute(k,v); for (const child of data.children || []) n.appendChild(build(child)); return n; }
  function fragment(html) { return JSON.parse(parse(html, true)).map(build); }
  function submit(form, button) {
    if (!form || form.localName !== 'form') return;
    if ((form.method || 'get').toLowerCase() !== 'get') throw Error('Only GET forms are supported');
    const pairs = [];
    for (const n of form.elements) {
      if (n.disabled || !n.name || (['submit','button','reset'].includes(n.type) && n !== button) || (['radio','checkbox'].includes(n.type) && !n.checked)) continue;
      const values = n.localName === 'select' ? n.options.filter(o => o.selected).map(o => o.value) : [n.value || (['radio','checkbox'].includes(n.type) ? 'on' : '')];
      values.forEach(v => pairs.push(encodeURIComponent(n.name) + '=' + encodeURIComponent(v)));
    }
    navigate(resolve(document.URL, form.action || document.URL).split(/[?#]/)[0] + '?' + pairs.join('&'));
  }
  function navigate(url) { const full = resolve(document.URL, String(url)); if (!/^https?:\/\//i.test(full)) throw Error('Unsupported navigation scheme'); navigation = full; }
  function activate(node) {
    if (node.disabled || !node.dispatchEvent(new Event('click', {bubbles:true,cancelable:true}))) return;
    if (node.localName === 'a') { const href = node.getAttribute('href') || ''; if (/^javascript:/i.test(href)) (0,eval)(href.slice(11)); else if (href) navigate(href); }
    else if (node.type === 'reset') { node.form?.elements.forEach(n => { delete n._value; delete n._checked; delete n._selected; }); }
    else if (node.form && (node.type === 'submit' || (node.localName === 'button' && !node.type))) node.form.requestSubmit(node);
  }
  const voidTags = new Set(['area','base','br','col','embed','hr','img','input','link','meta','param','source','track','wbr']);
  function serialize(n, output) {
    if (n.nodeType === 3) return escape(n._text);
    if (n.nodeType === 9) return n.childNodes.map(c => serialize(c,output)).join('');
    // No script code or executable event attributes is exported to the UI/saved document.
    if (output && ['script','style','template','noscript','iframe','object','embed'].includes(n.localName)) return '';
    const attrs = {...n._attrs};
    if (n._value !== undefined && n.localName !== 'textarea') attrs.value = n._value;
    if (n._checked !== undefined) { delete attrs.checked; if (n._checked) attrs.checked = ''; }
    if (n._selected !== undefined) { delete attrs.selected; if (n._selected) attrs.selected = ''; }
    if (n.style.display === 'none' || n.style.visibility === 'hidden') attrs.hidden = '';
    if (output) {
      for (const k of Object.keys(attrs)) if (k.startsWith('on') || k.startsWith('data-phone-')) delete attrs[k];
      attrs['data-phone-node'] = n._id;
      const custom = !['html','body','form'].includes(n.localName) && (n.onclick || n.hasAttribute('onclick') || n.getAttribute('role') === 'button');
      if (n.localName === 'a' && n.hasAttribute('href') || n.localName === 'button' || (n.localName === 'input' && ['submit','button','reset'].includes(n.type)) || custom) {
        const target = n.localName === 'a' && /^https?:\/\//i.test(n.href) ? '?target=' + encodeURIComponent(n.href) : '';
        attrs['data-phone-action'] = 'about:script/' + session + '/' + n._id + target;
      }
    }
    const start = '<' + n.localName + Object.entries(attrs).map(([k,v]) => ' ' + k + '="' + escape(v) + '"').join('') + '>';
    if (voidTags.has(n.localName)) return start;
    return start + (n.localName === 'textarea' && n._value !== undefined ? escape(n._value) : n.childNodes.map(c => serialize(c,output)).join('')) + '</' + n.localName + '>';
  }
  globalThis.window = globalThis; globalThis.self = globalThis; globalThis.top = globalThis; globalThis.parent = globalThis;
  const events = new Target();
  globalThis.addEventListener = events.addEventListener.bind(events); globalThis.removeEventListener = events.removeEventListener.bind(events);
  globalThis.dispatchEvent = event => { const result = events.dispatchEvent(event); const fn = globalThis['on' + event.type]; if (typeof fn === 'function') fn.call(window,event); return result; };
  globalThis.Event = Event; globalThis.CustomEvent = class extends Event { constructor(t,o) { super(t,o); this.detail = o?.detail; }};
  globalThis.Node = Node; globalThis.Element = Node; globalThis.HTMLElement = Node;
  globalThis.navigator = Object.freeze({userAgent:'TA1278 Text/0.2',language:'zh-CN',languages:['zh-CN','en'],cookieEnabled:false,onLine:true});
  globalThis.console = Object.freeze({log(){},warn(){},error(){},info(){},debug(){}});
  globalThis.performance = Object.freeze({now: () => Date.now()});
  let nextTimer = 0;
  globalThis.setTimeout = (fn, delay = 0, ...args) => { if (timers.size >= 32) throw Error('Timer limit'); const id = ++nextTimer; timers.set(id,{due:Date.now()+Math.max(0,Number(delay)||0), fn:() => typeof fn === 'function' ? fn(...args) : (0,eval)(String(fn))}); return id; };
  globalThis.clearTimeout = id => timers.delete(id);
  globalThis.setInterval = (fn,delay,...args) => { let id; const repeat = () => { fn(...args); if (timers.has(id)) timers.set(id,{due:Date.now()+Math.max(20,Number(delay)||0),fn:repeat,interval:true}); }; id=setTimeout(repeat,Math.max(20,Number(delay)||0)); timers.get(id).interval=true; return id; };
  globalThis.clearInterval = clearTimeout;
  globalThis.queueMicrotask = fn => Promise.resolve().then(fn);
  globalThis.requestAnimationFrame = fn => setTimeout(() => fn(performance.now()),16); globalThis.cancelAnimationFrame = clearTimeout;
  globalThis.fetch = (url, options = {}) => Promise.resolve().then(() => {
    if ((options.method || 'GET').toUpperCase() !== 'GET' || options.body !== undefined) throw Error('Only same-origin GET fetch is supported');
    const r = JSON.parse(resource(resolve(document.URL,String(url)), 'fetch'));
    if (r.error) throw TypeError(r.error);
    return Object.freeze({ok:r.status>=200&&r.status<300,status:r.status,url:r.url,text:()=>Promise.resolve(r.body),json:()=>Promise.resolve().then(()=>JSON.parse(r.body))});
  });
  globalThis.URLSearchParams = class {
    constructor(value = '') { this._pairs = typeof value === 'string' ? value.replace(/^\?/,'').split('&').filter(Boolean).map(p => { const i=p.indexOf('=');return [i<0?p:p.slice(0,i),i<0?'':p.slice(i+1)].map(s=>decodeURIComponent(s.replace(/\+/g,' '))); }) : Array.isArray(value) ? value.map(p=>p.map(String)) : Object.entries(value).map(([k,v])=>[k,String(v)]); }
    append(k,v) { this._pairs.push([String(k),String(v)]); }
    get(k) { return this._pairs.find(p=>p[0]===String(k))?.[1] ?? null; }
    getAll(k) { return this._pairs.filter(p=>p[0]===String(k)).map(p=>p[1]); }
    has(k) { return this.get(k)!==null; }
    delete(k) { this._pairs=this._pairs.filter(p=>p[0]!==String(k)); }
    set(k,v) { this.delete(k);this.append(k,v); }
    entries() { return this._pairs[Symbol.iterator](); } [Symbol.iterator]() { return this.entries(); }
    forEach(fn) { this._pairs.forEach(([k,v])=>fn(v,k,this)); }
    toString() { return this._pairs.map(p=>p.map(v=>encodeURIComponent(v).replace(/%20/g,'+')).join('=')).join('&'); }
  };
  globalThis.URL = class {
    constructor(url,base=document.URL) { this.href=resolve(String(base),String(url)); if(!this.href)throw TypeError('Invalid URL'); }
    toString() { return this.href; } toJSON() { return this.href; }
    get searchParams() { return new URLSearchParams(this.search); }
  };
  for(const key of ['origin','protocol','host','hostname','pathname','search','hash']) Object.defineProperty(URL.prototype,key,{get(){return JSON.parse(resolve(this.href,'',true))[key];}});
  globalThis.XMLHttpRequest = class extends Target {
    constructor(){super();this.readyState=0;this.status=0;this.responseText='';this.response=null;}
    open(method,url,async=true){if(String(method).toUpperCase()!=='GET'||async===false)throw Error('Only asynchronous GET is supported');this._url=url;this.readyState=1;this.dispatchEvent(new Event('readystatechange'));}
    send(body){if(body!=null)throw Error('Request bodies are not supported');fetch(this._url).then(r=>r.text().then(text=>{if(this._aborted)return;this.status=r.status;this.responseText=text;this.response=this.responseType==='json'?JSON.parse(text):text;this.readyState=4;this.dispatchEvent(new Event('readystatechange'));this.dispatchEvent(new Event('load'));this.dispatchEvent(new Event('loadend'));})).catch(()=>{this.readyState=4;this.dispatchEvent(new Event('error'));this.dispatchEvent(new Event('loadend'));});}
    abort(){this._aborted=true;this.readyState=0;} setRequestHeader(){throw Error('Custom request headers are not supported');}
  };
  const location = {get href(){return document.URL;},set href(v){navigate(v);},assign:navigate,replace:navigate,reload(){navigation=document.URL;},toString(){return document.URL;}};
  for (const key of ['origin','protocol','host','hostname','pathname','search','hash']) Object.defineProperty(location,key,{get(){const u=JSON.parse(resolve(document.URL,'',true)); return u[key];}});
  Object.defineProperty(globalThis,'location',{get:()=>location,set:navigate});
  // Internal control object is captured by the host; scripts cannot replace it.
  Object.defineProperty(globalThis,'__phone',{value:Object.freeze({
    init(data,id,url) {
      session=id; document = new Node('#document'); globalThis.document=document; document.URL=url; document.readyState='loading';
      document.createElement = tag => new Node(String(tag)); document.createTextNode = text => new Node('',text);
      document.createDocumentFragment = () => new Node('#document'); document.getElementById = id => document.querySelectorAll('[id]').find(n=>n.id===String(id)) || null;
      document.getElementsByName = name => document.querySelectorAll('[name]').filter(n=>n.name===String(name));
      for (const item of data) document.appendChild(build(item));
      for (const [key,selector] of [['documentElement','html'],['head','head'],['body','body']]) Object.defineProperty(document,key,{get:()=>document.querySelector(selector)});
      Object.defineProperty(document,'title',{get:()=>document.querySelector('title')?.textContent || '',set(v){let n=document.querySelector('title');if(!n){n=document.createElement('title');(document.head||document.documentElement).appendChild(n);}n.textContent=v;}});
      Object.defineProperty(document,'location',{get:()=>location,set:navigate});
      document.write = (...s) => { const owner=writing?.parentNode || document.body; if(owner) for(const n of fragment(s.join(''))) owner.insertBefore(n,writing?.nextSibling || null); };
      document.writeln = (...s) => document.write(...s,'\n');
      document.querySelectorAll('select').forEach(n=>{if(!n.multiple&&!n.options.some(o=>o.selected)&&n.options.length)n.options[0].selected=true;});
    },
    scripts() { return JSON.stringify(document.querySelectorAll('script').filter(n=>!n._executed).slice(0,32).map(n=>{n._executed=true;return {id:n._id,src:n.getAttribute('src'),type:n.type,code:n.textContent};})); },
    current(id) { writing=nodes.get(id);document.currentScript=writing; },
    ready() { writing=null;document.currentScript=null;document.readyState='interactive';document.dispatchEvent(new Event('DOMContentLoaded',{bubbles:true}));document.readyState='complete';dispatchEvent(new Event('load'));document.body?.dispatchEvent(new Event('load')); },
    timers() { let ran=0; for(const [id,t] of timers) if(t.due<=Date.now()){ if(!t.interval)timers.delete(id);t.fn();if(++ran>=32)break; } return ran; },
    sync(values) { for(const v of values){const n=nodes.get(v.id);if(!n)continue;let changed=false;if(v.value!==undefined&&n.value!==v.value){n.value=v.value;changed=true;}if(v.checked!==undefined&&n.checked!==v.checked){n.checked=v.checked;changed=true;}if(v.selected){n.options.forEach((o,i)=>{if(o.selected!==!!v.selected[i])changed=true;o.selected=!!v.selected[i];});}if(changed){n.dispatchEvent(new Event('input',{bubbles:true}));n.dispatchEvent(new Event('change',{bubbles:true}));}} },
    click(id) { const n=nodes.get(id);if(!n||!n.isConnected)throw Error('Stale DOM target');activate(n); },
    snapshot() { const html=serialize(document,true);if(html.length>MAX_TEXT)throw Error('DOM output limit');const next=navigation;navigation=null;return JSON.stringify({html,next}); }
  })});
})();
