use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen(inline_js = r#"
const KEY = 'great-idea-swap-machine-v1';
const APP_VERSION = '1.2.2';
const CATEGORIES = ['🚀 Almost Scientific','🤖 Questionably Useful Technology','🍕 Food That Shouldn’t Exist','🏠 Ridiculous Household Inventions','🎬 Impossible Podcast or Movie Ideas','🧸 Colin-and-Luan Approved Barter Businesses'];
const AWARDS = ['Most Brilliantly Ridiculous','Strangely Marketable','Most Likely to Concern NASA','Best Idea Improved by Someone Else','Idea We Accidentally Need'];
const $ = id => document.getElementById(id);
const esc = value => String(value ?? '').replace(/[&<>'"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
const uid = () => crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
const params = new URLSearchParams(location.search);
const DEFAULT_API = 'https://great-idea-swap-backend.mikegyver.workers.dev';
const api = params.get('local') === '1' ? '' : (params.get('api') || DEFAULT_API).replace(/\/$/, '');
const room = (params.get('room') || 'family-room').trim().toLowerCase().replace(/[^a-z0-9-]/g,'-').slice(0,48) || 'family-room';
let state;
let held = null;
let revealOpen = false;
let healthTimer = null;
let serviceState = api ? 'checking' : 'local';
let lastHealthy = null;
const blank = () => ({ ideas: [], votes: {}, activity: [], created: new Date().toISOString() });
const seeds = () => [
  ['Machine Spirit','🚀 Almost Scientific','A telescope that gets embarrassed when it discovers a new planet.'],
  ['Machine Spirit','🤖 Questionably Useful Technology','A doorbell that negotiates with delivery robots before letting them approach.'],
  ['Machine Spirit','🍕 Food That Shouldn’t Exist','A breakfast taco that changes fillings based on the weather forecast.'],
  ['Machine Spirit','🏠 Ridiculous Household Inventions','A laundry basket that files missing-sock incident reports.'],
  ['Machine Spirit','🎬 Impossible Podcast or Movie Ideas','A detective podcast narrated by the clues that were overlooked.'],
  ['Machine Spirit','🧸 Colin-and-Luan Approved Barter Businesses','A homework help desk that accepts only plushies and grape candy.']
].map(([owner,category,text]) => ({ id:uid(), owner, category, chain:[{author:owner,text}], claimed:null, created:Date.now() }));
function normalize(raw){
  const value=raw&&typeof raw==='object'?raw:blank();
  value.ideas=Array.isArray(value.ideas)?value.ideas:[];
  value.votes=value.votes&&typeof value.votes==='object'&&!Array.isArray(value.votes)?value.votes:{};
  value.activity=Array.isArray(value.activity)?value.activity:[];
  value.created=typeof value.created==='string'?value.created:new Date().toISOString();
  return value;
}
function localLoad(){ try { return normalize(JSON.parse(localStorage.getItem(KEY))); } catch { return blank(); } }
function localSave(){ localStorage.setItem(KEY, JSON.stringify(state)); }
function audit(event,player='Machine',details='',ideaId=null){
  state.activity ||= [];
  state.activity.push({timestamp:new Date().toISOString(),player:String(player||'Anonymous').slice(0,40),event,idea_id:ideaId,details:String(details||'').slice(0,500)});
  if(state.activity.length>5000)state.activity=state.activity.slice(-5000);
}
function name(){ return $('player-name').value.trim(); }
function say(text, error=false){ $('message').textContent=text; $('message').classList.toggle('error',error); }
function validateLength(inputId,counterId,buttonId,max=280){
  const length=$(inputId).value.length,remaining=max-length,over=remaining<0;
  $(counterId).textContent=over?`${length}/${max} — remove ${-remaining}`:`${length}/${max}`;
  $(counterId).classList.toggle('over',over);$(buttonId).disabled=over;
  return !over;
}
function service(status,detail=''){
  serviceState=status;
  const el=$('service-status'); if(!el)return;
  const labels={checking:'CHECKING BACKEND',active:'DURABLE OBJECT ACTIVE',offline:'BACKEND OFFLINE',degraded:'BACKEND DEGRADED',local:'LOCAL MODE'};
  el.className=`service-status ${status}`;
  el.querySelector('.service-label').textContent=labels[status]||status.toUpperCase();
  el.querySelector('.service-detail').textContent=detail||(status==='local'?'No cloud service required.':`Room: ${room}`);
}
async function checkHealth(showMessage=false){
  if(!api){service('local');return true;}
  service(serviceState==='active'?'active':'checking',`Room: ${room} • contacting service…`);
  try{
    const controller=new AbortController(),timeout=setTimeout(()=>controller.abort(),7000);
    const response=await fetch(`${api}/api/health?room=${encodeURIComponent(room)}`,{cache:'no-store',signal:controller.signal});clearTimeout(timeout);
    if(!response.ok)throw new Error(`HTTP ${response.status}`);
    const data=await response.json();lastHealthy=new Date();
    const writable=data.writable!==false;
    service(writable?'active':'degraded',`Room: ${room} • ${data.ideas??0} ideas • checked ${lastHealthy.toLocaleTimeString()}`);
    if(showMessage)say(writable?'Backend connection is healthy.':'Backend is reachable but currently read-only.',!writable);
    return writable;
  }catch(error){
    const prior=lastHealthy?` • last connected ${lastHealthy.toLocaleTimeString()}`:'';
    service('offline',`Room: ${room}${prior}`);
    if(showMessage)say(`Backend health check failed: ${error.message||error}. Your displayed session remains intact.`,true);
    return false;
  }
}
function scheduleHealth(){
  clearInterval(healthTimer);healthTimer=null;
  if(api&&!document.hidden)healthTimer=setInterval(()=>checkHealth(false),30000);
}
async function remote(action,payload={}){
  try{
    const response=await fetch(`${api}/api/action`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({room,action,payload})});
    const data=await response.json().catch(()=>({}));
    if(!response.ok)throw new Error(data.error||`Multiplayer service returned HTTP ${response.status}.`);
    if(data.state){state=normalize(data.state);localSave();}lastHealthy=new Date();service('active',`Room: ${room} • connected ${lastHealthy.toLocaleTimeString()}`);return data;
  }catch(error){service('offline',`Room: ${room}${lastHealthy?` • last connected ${lastHealthy.toLocaleTimeString()}`:''}`);throw error;}
}
async function act(action,payload,localFn){
  try { const result=api ? await remote(action,payload) : localFn(); render(); return result; }
  catch(error){ say(error.message || String(error),true); throw error; }
}
function options(){ return CATEGORIES.map(c=>`<option>${esc(c)}</option>`).join(''); }
function ensureIdentity(){ const n=name(); if(!n){ say('Enter your player name before approaching the machine.',true); return ''; }if(['__proto__','prototype','constructor'].includes(n.toLowerCase())){say('That reserved machine name cannot be used.',true);return '';} sessionStorage.setItem('idea-swap-name',n); return n; }
function submitIdea(){
  const owner=ensureIdentity(), text=$('idea-input').value.trim(), category=$('category').value;
  if(!owner)return;if(!validateLength('idea-input','idea-count','submit-idea')){say('That idea exceeds the 280-character machine limit. Shorten it before depositing; nothing has been clipped.',true);return;} if(text.length<10){say('Give the machine at least 10 characters of wonderfully strange material.',true);return;}
  act('submit',{owner,text,category},()=>{const idea={id:uid(),owner,category,chain:[{author:owner,text}],claimed:null,created:Date.now()};state.ideas.push(idea);audit('idea_submitted',owner,category,idea.id);localSave();return{};}).then(()=>{$('idea-input').value='';validateLength('idea-input','idea-count','submit-idea');say('Idea accepted. The machine promises not to return it to you.');});
}
function pullIdea(){
  const player=ensureIdentity(); if(!player)return;
  if(held){say('You already have an idea. Twist it or put it back before pulling again.',true);return;}
  act('pull',{player},()=>{
    const candidates=state.ideas.filter(i=>i.owner.toLowerCase()!=player.toLowerCase()&&!i.claimed&&i.chain.at(-1).author.toLowerCase()!=player.toLowerCase());
    if(!candidates.length)throw new Error('No eligible idea is waiting. Invite another thinker or submit ideas under another player.');
    const idea=candidates[Math.floor(Math.random()*candidates.length)]; idea.claimed=player; held=idea.id;audit('lever_pulled',player,'An eligible idea was randomly selected.',idea.id);audit('idea_assigned',player,idea.category,idea.id);localSave(); return{held:idea.id};
  }).then(result=>{held=result?.held||held;say('CLUNK! The machine has issued someone else’s idea. Add an unexpected twist.');render();});
}
function passTwist(){
  const player=ensureIdentity(), text=$('twist-input').value.trim(); if(!player)return;
  if(!held){say('Pull the lever before attempting to twist reality.',true);return;}if(!validateLength('twist-input','twist-count','pass-twist')){say('That twist exceeds the 280-character machine limit. Shorten it before passing; nothing has been clipped.',true);return;} if(text.length<8){say('The twist needs at least 8 characters.',true);return;}
  act('twist',{player,id:held,text},()=>{const idea=state.ideas.find(i=>i.id===held);if(!idea||idea.claimed!==player)throw new Error('That idea is no longer assigned to this player.');idea.chain.push({author:player,text});idea.claimed=null;audit('twist_added',player,text,idea.id);localSave();return{};}).then(()=>{held=null;$('twist-input').value='';validateLength('twist-input','twist-count','pass-twist');say('Twist attached. The idea is back inside the machine, noticeably weirder.');render();});
}
function releaseHeld(){
  if(!held)return; const player=name();
  act('release',{player,id:held},()=>{const idea=state.ideas.find(i=>i.id===held);if(idea&&idea.claimed===player){idea.claimed=null;audit('idea_returned',player,'Returned without a twist.',idea.id);}localSave();return{};}).then(()=>{held=null;say('Idea returned to the machine without modification.');render();});
}
function vote(award,id){
  const player=ensureIdentity();if(!player)return;
  act('vote',{player,award,id},()=>{state.votes[award] ||= {};state.votes[award][player]=id;audit('vote_cast',player,award,id);localSave();return{};}).then(()=>say(`Vote recorded for “${award}.”`));
}
function reset(){ if(api){say('Reset is disabled in Public Multiplayer Mode.',true);return;}if(!confirm('Erase every local idea, twist, vote, and activity entry on this device?'))return;state=blank();state.ideas=seeds();audit('machine_reset',name()||'Machine','Six starter ideas installed.');localSave();held=null;revealOpen=false;say('The local machine has been reset with six starter ideas.');render(); }
function finalConcept(i){return i.chain.map((s,n)=>`${n?'Twist':'Original'}: ${s.text}`).join(' → ')}
function winner(award){const votes=state.votes[award]||{};const counts={};Object.values(votes).forEach(id=>counts[id]=(counts[id]||0)+1);const pair=Object.entries(counts).sort((a,b)=>b[1]-a[1])[0];return pair?`${state.ideas.find(i=>i.id===pair[0])?.chain[0].text||'Unknown'} (${pair[1]})`:'No votes yet';}
function downloadJson(data,filename){
  const blob=new Blob([JSON.stringify(data,null,2)],{type:'application/json'}),url=URL.createObjectURL(blob),link=document.createElement('a');
  link.href=url;link.download=filename;document.body.appendChild(link);link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
function safeText(value,max,label){if(typeof value!=='string'||!value.trim()||value.length>max)throw new Error(`${label} is missing or too long.`);return value.trim();}
function sanitizeSession(raw){
  const source=raw?.session||raw;
  if(!source||typeof source!=='object'||!Array.isArray(source.ideas))throw new Error('This is not a Great Idea Swap Machine session.');
  if(source.ideas.length>2000)throw new Error('Import rejected: more than 2,000 ideas.');
  const ids=new Set();
  const ideas=source.ideas.map((idea,index)=>{
    if(!idea||typeof idea!=='object'||!Array.isArray(idea.chain)||!idea.chain.length||idea.chain.length>100)throw new Error(`Idea ${index+1} has an invalid journey.`);
    const id=safeText(idea.id||uid(),100,`Idea ${index+1} ID`);if(ids.has(id))throw new Error('Import rejected: duplicate idea ID.');ids.add(id);
    const owner=safeText(idea.owner,40,`Idea ${index+1} owner`),category=safeText(idea.category,90,`Idea ${index+1} category`);
    const chain=idea.chain.map((step,n)=>({author:safeText(step?.author,40,`Idea ${index+1} contributor`),text:safeText(step?.text,280,`Idea ${index+1} step ${n+1}`)}));
    return{id,owner,category,chain,claimed:typeof idea.claimed==='string'?idea.claimed.slice(0,40):null,created:Number.isFinite(Number(idea.created))?Number(idea.created):Date.now()};
  });
  const votes={};
  for(const award of AWARDS){const incoming=source.votes?.[award];if(!incoming||typeof incoming!=='object'||Array.isArray(incoming))continue;votes[award]={};for(const [player,id] of Object.entries(incoming).slice(0,5000)){if(!['__proto__','prototype','constructor'].includes(player.toLowerCase())&&ids.has(String(id)))votes[award][String(player).slice(0,40)]=String(id);}}
  const activity=(Array.isArray(source.activity)?source.activity:[]).slice(-5000).map(entry=>({timestamp:typeof entry?.timestamp==='string'?entry.timestamp.slice(0,40):new Date().toISOString(),player:typeof entry?.player==='string'?entry.player.slice(0,40):'Unknown',event:typeof entry?.event==='string'?entry.event.slice(0,60):'legacy_entry',idea_id:entry?.idea_id&&ids.has(String(entry.idea_id))?String(entry.idea_id):null,details:typeof entry?.details==='string'?entry.details.slice(0,500):''}));
  return{ideas,votes,activity,created:typeof source.created==='string'?source.created.slice(0,40):new Date().toISOString()};
}
function exportSession(){
  if(!api){audit('session_exported',name()||'Machine','Complete session JSON exported.');localSave();render();}
  downloadJson({app:'The Great Idea Swap Machine',version:APP_VERSION,exported_at:new Date().toISOString(),mode:api?'public-multiplayer':'pass-the-phone',session:state},`great-idea-swap-session-${new Date().toISOString().slice(0,10)}.json`);say('Complete session JSON exported.');
}
function exportActivity(){
  if(!api){audit('activity_exported',name()||'Machine','Activity-only JSON exported.');localSave();render();}
  downloadJson({app:'The Great Idea Swap Machine',version:APP_VERSION,exported_at:new Date().toISOString(),mode:api?'public-multiplayer':'pass-the-phone',activity:state.activity||[]},`great-idea-swap-activity-${new Date().toISOString().slice(0,10)}.json`);say('Activity log JSON exported.');
}
async function importSession(file){
  try{
    if(!file)return;if(file.size>10*1024*1024)throw new Error('Import rejected: JSON file exceeds 10 MB.');
    const imported=sanitizeSession(JSON.parse(await file.text()));
    if(!confirm(`Replace the current session with ${imported.ideas.length} imported ideas and ${imported.activity.length} activity entries?`))return;
    const player=name()||'Machine';
    if(api){await act('import',{player,session:imported},()=>({}));}
    else{state=imported;audit('session_imported',player,`${file.name}: ${imported.ideas.length} ideas.`);localSave();held=null;revealOpen=false;render();}
    say(`Session imported safely: ${imported.ideas.length} ideas and ${imported.activity.length} previous activity entries.`);
  }catch(error){say(`Import failed: ${error.message||error}`,true);}
  finally{$('import-file').value='';}
}
function render(){
  const player=name(); $('mode').textContent=api?`PUBLIC • ${room.toUpperCase()}`:'PASS-THE-PHONE';
  const assigned=state.ideas.find(i=>i.claimed===player); held=assigned?.id||null;
  const mine=state.ideas.filter(i=>i.owner.toLowerCase()===player.toLowerCase()).length;
  $('stat-ideas').textContent=state.ideas.length;$('stat-twists').textContent=state.ideas.reduce((n,i)=>n+Math.max(0,i.chain.length-1),0);$('stat-mine').textContent=mine;$('stat-waiting').textContent=state.ideas.filter(i=>!i.claimed).length;
  const idea=state.ideas.find(i=>i.id===held);
  $('held').classList.toggle('show',!!idea);
  if(idea){$('held-category').textContent=idea.category;$('held-original').textContent=idea.chain[0].text;$('held-chain').textContent=`Journey so far: ${idea.chain.length} contribution${idea.chain.length===1?'':'s'}. Original author hidden until reveal.`;}
  $('reveal-body').hidden=!revealOpen;
  $('reveal-button').textContent=revealOpen?'Close the Vault':'Reveal Every Journey + Vote';
  if(revealOpen){
    $('journeys').innerHTML=state.ideas.map((i,index)=>`<article class="journey"><span class="category">${esc(i.category)}</span><h3>Idea ${index+1}: ${esc(i.chain[0].text)}</h3>${i.chain.map((s,n)=>`<p class="step"><strong>${n?'Twist':'Original'} — ${esc(s.author)}</strong><br>${esc(s.text)}</p>`).join('')}<p class="chain-mini"><strong>Final concept:</strong> ${esc(finalConcept(i))}</p></article>`).join('')||'<p>No ideas yet.</p>';
    $('votes').innerHTML=AWARDS.map(a=>`<div class="vote"><label>${esc(a)}<select data-award="${esc(a)}"><option value="">Choose an idea…</option>${state.ideas.map((i,n)=>`<option value="${i.id}" ${state.votes[a]?.[player]===i.id?'selected':''}>Idea ${n+1}: ${esc(i.chain[0].text.slice(0,55))}</option>`).join('')}</select></label><small>Leader: ${esc(winner(a))}</small></div>`).join('');
  }
  const recent=(state.activity||[]).slice(-14).reverse();
  $('activity-feed').innerHTML=recent.map(entry=>`<li><time>${esc(new Date(entry.timestamp).toLocaleString())}</time><strong>${esc(entry.player)}</strong><span>${esc(entry.event.replaceAll('_',' '))}${entry.details?`: ${esc(entry.details)}`:''}</span></li>`).join('')||'<li class="empty-log">No activity recorded yet.</li>';
  $('activity-total').textContent=`${(state.activity||[]).length} entr${(state.activity||[]).length===1?'y':'ies'}`;
}
export async function initIdeaSwapMachine(){
  $('category').innerHTML=options();$('player-name').value=sessionStorage.getItem('idea-swap-name')||'';
  if(api){service('checking',`Room: ${room}`);try{const response=await fetch(`${api}/api/state?room=${encodeURIComponent(room)}`,{cache:'no-store'});if(!response.ok)throw new Error(`HTTP ${response.status}`);const data=await response.json();state=normalize(data.state||data);localSave();lastHealthy=new Date();service('active',`Room: ${room} • connected ${lastHealthy.toLocaleTimeString()}`);say('Public Multiplayer connected. Ideas can travel between devices.');}catch(e){state=localLoad();service('offline',`Room: ${room} • no cloud connection`);say(`Public Multiplayer could not connect: ${e.message}. The last local snapshot is displayed without cloud writes.`,true);}}
  else{state=localLoad();if(!state.ideas.length){state.ideas=seeds();audit('starter_ideas_loaded','Machine Spirit','Six starter ideas installed.');localSave();}say('Pass-the-Phone Mode is ready. Six starter ideas are already rattling inside.');}
  $('check-service').onclick=()=>checkHealth(true);
  document.addEventListener('visibilitychange',()=>{scheduleHealth();if(!document.hidden)checkHealth(false);});scheduleHealth();
  $('save-name').onclick=()=>{if(ensureIdentity()){say(`Player “${name()}” is at the controls.`);render();}};
  $('submit-idea').onclick=submitIdea;$('pull-lever').onclick=pullIdea;$('pass-twist').onclick=passTwist;$('release-held').onclick=releaseHeld;
  $('reveal-button').onclick=()=>{revealOpen=!revealOpen;if(revealOpen&&!api){audit('reveal_opened',name()||'Machine','End-of-day journey vault opened.');localSave();}render();};$('reset-machine').onclick=reset;
  $('export-session').onclick=exportSession;$('export-activity').onclick=exportActivity;$('import-session').onclick=()=>$('import-file').click();
  $('import-file').onchange=e=>importSession(e.target.files?.[0]);
  $('idea-input').oninput=()=>validateLength('idea-input','idea-count','submit-idea');
  $('twist-input').oninput=()=>validateLength('twist-input','twist-count','pass-twist');
  $('votes').onchange=e=>{if(e.target.dataset.award&&e.target.value)vote(e.target.dataset.award,e.target.value);};
  render();
}
"#)]
extern "C" { #[wasm_bindgen(js_name = initIdeaSwapMachine)] fn init_machine(); }

#[function_component(App)]
fn app() -> Html {
    use_effect(|| { init_machine(); || () });
    html! {
      <main class="app">
        <header class="hero"><div><p class="eyebrow">{"MIKEGYVER STUDIO • WONDERFULLY STRANGE THINKING LAB"}</p><h1>{"The Great Idea Swap Machine"}</h1><p class="subtitle">{"Leave an idea. Take an idea. Make it wonderfully weirder."}</p></div><span id="mode" class="badge">{"STARTING…"}</span></header>
        <p id="message" class="message">{"Warming up the gears…"}</p>
        <section id="service-status" class="service-status checking"><span class="service-light"></span><div><strong class="service-label">{"CHECKING BACKEND"}</strong><small class="service-detail">{"Starting telemetry…"}</small></div><button id="check-service" class="service-check">{"Check now"}</button></section>
        <section class="panel"><h2>{"Who is pulling the lever?"}</h2><p class="panel-intro">{"Use the same name for your turn. The machine will never hand you your own original idea."}</p><div class="identity"><input id="player-name" maxlength="40" placeholder="Player name or nickname"/><button id="save-name" class="button secondary">{"Take the Controls"}</button></div></section>
        <section class="machine-grid">
          <div class="panel"><h2>{"1. Feed the machine"}</h2><p class="panel-intro">{"Contribute one gloriously strange starting point."}</p><label class="field">{"Category"}<select id="category"></select></label><label class="field">{"Original idea"}<textarea id="idea-input" placeholder="A coffee mug that warns you before someone schedules a Monday meeting…"></textarea></label><div class="submit-row"><span id="idea-count" class="counter">{"0/280"}</span><button id="submit-idea" class="button primary">{"Deposit Idea 💡"}</button></div></div>
          <div class="panel"><h2>{"2. Tempt the machine"}</h2><p class="panel-intro">{"Pull the lever and accept whatever questionable brilliance appears."}</p><div class="lever-box"><div><div class="lever"></div><button id="pull-lever" class="button primary lever-button">{"PULL THE LEVER 🔄"}</button></div></div></div>
        </section>
        <section id="held" class="held"><span id="held-category" class="category"></span><p id="held-original" class="idea-text"></p><p id="held-chain" class="chain-mini"></p><label class="field">{"3. Add one unexpected twist"}<textarea id="twist-input" placeholder="It communicates only through dramatic movie-trailer narration…"></textarea></label><div class="twist-actions"><span id="twist-count" class="counter">{"0/280"}</span><button id="pass-twist" class="button primary">{"Attach Twist + Pass It On"}</button><button id="release-held" class="button secondary">{"Put It Back"}</button></div></section>
        <section class="panel"><h2>{"Machine telemetry"}</h2><p class="panel-intro">{"A responsible dashboard for deeply irresponsible innovation."}</p><div class="stats"><div class="stat"><span>{"IDEAS"}</span><strong id="stat-ideas">{"0"}</strong></div><div class="stat"><span>{"TWISTS"}</span><strong id="stat-twists">{"0"}</strong></div><div class="stat"><span>{"YOUR IDEAS"}</span><strong id="stat-mine">{"0"}</strong></div><div class="stat"><span>{"WAITING"}</span><strong id="stat-waiting">{"0"}</strong></div></div></section>
        <section class="panel"><div class="reveal-head"><div><h2>{"End-of-Day Reveal Vault"}</h2><p class="panel-intro">{"Open every journey, reveal the contributors, and vote for glorious nonsense."}</p></div><button id="reveal-button" class="button secondary">{"Reveal Every Journey + Vote"}</button></div><div id="reveal-body" hidden=true><div id="journeys" class="journeys"></div><h2>{"🏆 Cast your votes"}</h2><div id="votes" class="vote-grid"></div></div></section>
        <section class="panel"><div class="log-title"><div><h2>{"Machine Activity Ledger"}</h2><p class="panel-intro">{"A timestamped record of deposits, pulls, twists, returns, votes, reveals, exports, and imports."}</p></div><span id="activity-total" class="badge">{"0 entries"}</span></div><ol id="activity-feed" class="activity-feed"></ol><div class="toolbar"><button id="export-session" class="button primary">{"Export Complete Session JSON"}</button><button id="export-activity" class="button secondary">{"Export Activity Log JSON"}</button><button id="import-session" class="button secondary">{"Import Session JSON"}</button><input id="import-file" class="file-hidden" type="file" accept=".json,application/json"/><button id="reset-machine" class="button danger">{"Reset Local Machine"}</button></div><p class="mode-note">{"Imports are validated before replacing the current session. Export a backup first when the machine contains irreplaceable nonsense. Pass-the-Phone stores everything on this device; add ?api=https://YOUR-WORKER for the optional multiplayer service."}</p></section>
        <p class="slogan">{"Sometimes a wild idea does not need to be practical—it just needs to inspire the next great idea."}</p>
      </main>
    }
}

fn main(){ yew::Renderer::<App>::new().render(); }
