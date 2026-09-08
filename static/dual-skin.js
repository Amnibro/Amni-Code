(function(g){
var SK='amni_skin',TK='amni-theme',OK=['braid','amniscient'];
function _ls(k,d){try{var v=localStorage.getItem(k);return v==null?d:v}catch(e){return d}}
function _sv(k,v){try{localStorage.setItem(k,v)}catch(e){}}
function getSkin(){var s=_ls(SK,'braid');return OK.indexOf(s)>=0?s:'braid'}
function getTone(){return _ls(TK,'dark')==='light'?'light':'dark'}
function _meta(skin,tone){var m=document.querySelector('meta[name=theme-color]');if(!m)return;m.setAttribute('content',skin==='amniscient'?(tone==='light'?'#F3F2EF':'#08090B'):(tone==='light'?'#FBFAF8':'#0A0B0E'))}
function _btns(skin,tone){var a=document.getElementById('amni-skin-braid');if(a)a.classList.toggle('on',skin==='braid');var b=document.getElementById('amni-skin-amniscient');if(b)b.classList.toggle('on',skin==='amniscient');var c=document.getElementById('amni-tone-toggle');if(c)c.textContent=tone==='dark'?'LIGHT':'DARK'}
function apply(skin,tone){
skin=OK.indexOf(skin)>=0?skin:'braid';tone=tone==='light'?'light':'dark';
var h=document.documentElement;h.setAttribute('data-skin',skin);h.setAttribute('data-theme',tone);
var body=document.body;if(body){body.setAttribute('data-skin',skin);body.setAttribute('data-tone',tone);body.classList.toggle('skin-braid',skin==='braid');body.classList.toggle('skin-amniscient',skin==='amniscient');body.classList.toggle('theme-min',skin==='amniscient');body.classList.toggle('theme-product',true)}
_sv(SK,skin);_sv(TK,tone);_meta(skin,tone);_btns(skin,tone);
try{g.dispatchEvent(new CustomEvent('amni-skin',{detail:{skin:skin,tone:tone}}))}catch(e){}
return {skin:skin,tone:tone}
}
function setSkin(s){return apply(s,getTone())}
function setTone(t){return apply(getSkin(),t)}
function toggleSkin(){return setSkin(getSkin()==='braid'?'amniscient':'braid')}
function toggleTone(){return setTone(getTone()==='dark'?'light':'dark')}
function boot(){return apply(getSkin(),getTone())}
g.AmniSkin={SK:SK,TK:TK,set:setSkin,get:getSkin,tone:getTone,setTone:setTone,toggle:toggleSkin,toggleTone:toggleTone,apply:apply,boot:boot};
if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',boot);else boot();
})(window);
