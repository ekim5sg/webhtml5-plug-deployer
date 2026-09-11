use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen(inline_js = r#"
const KEY = 'great-idea-swap-machine-v1';
const CATEGORIES = ['🚀 Almost Scientific','🤖 Questionably Useful Technology','🍕 Food That Shouldn’t Exist','🏠 Ridiculous Household Inventions','🎬 Impossible Podcast or Movie Ideas','🧸 Colin-and-Luan Approved Barter Businesses'];
const AWARDS = ['Most Brilliantly Ridiculous','Strangely Marketable','Most Likely to Concern NASA','Best Idea Improved by Someone Else','Idea We Accidentally Need'];
const $ = id => document.getElementById(id);
const esc = value => String(value ?? '').replace(/[&<>'"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'}[c]));
const uid = () => crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
const api = new URLSearchParams(location.search).get('api')?.replace(/\/$/, '') || '';
let state;
let held = null;
let revealOpen = false;
const blank = () => ({ ideas: [], votes: {}, created: new Date().toISOString() });
const seeds = () => [
  ['Machine Spirit','🚀 Almost Scientific','A telescope that gets embarrassed when it discovers a new planet.'],
  ['Machine Spirit','🤖 Questionably Useful Technology','A doorbell that negotiates with delivery robots before letting them approach.'],
  ['Machine Spirit','🍕 Food That Shouldn’t Exist','A breakfast taco that changes fillings based on the weather forecast.'],
  ['Machine Spirit','🏠 Ridiculous Household Inventions','A laundry basket that files missing-sock incident reports.'],
  ['Machine Spirit','🎬 Impossible Podcast or Movie Ideas','A detective podcast narrated by the clues that were overlooked.'],
  ['Machine Spirit','🧸 Colin-and-Luan Approved Barter Businesses','A homework help desk that accepts only plushies and grape candy.']
].map(([owner,category,text]) => ({ id:uid(), owner, category, chain:[{author:owner,text}], claimed:null, created:Date.now() }));
function localLoad(){ try { return JSON.parse(localStorage.getItem(KEY)) || blank(); } catch { return blank(); } }
function localSave(){ localStorage.setItem(KEY, JSON.stringify(state)); }
function name(){ return $('player-name').value.trim(); }
function say(text, error=false){ $('message').textContent=text; $('message').classList.toggle('error',error); }
async function remote(action,payload={}){
  const response=await fetch(`${api}/api/action`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({action,payload})});
  if(!response.ok) throw new Error(`Multiplayer service returned HTTP ${response.status}.`);
  const data=await response.json(); if(data.state) state=data.state; return data;
}
async function act(action,payload,localFn){
  try { const result=api ? await remote(action,payload) : localFn(); render(); return result; }
  catch(error){ say(error.message || String(error),true); throw error; }
}
function options(){ return CATEGORIES.map(c=>`<option>${esc(c)}</option>`).join(''); }
function ensureIdentity(){ const n=name(); if(!n){ say('Enter your player name before approaching the machine.',true); return ''; } sessionStorage.setItem('idea-swap-name',n); return n; }
function submitIdea(){
  const owner=ensureIdentity(), text=$('idea-input').value.trim(), category=$('category').value;
  if(!owner)return; if(text.length<10){say('Give the machine at least 10 characters of wonderfully strange material.',true);return;}
  act('submit',{owner,text,category},()=>{state.ideas.push({id:uid(),owner,category,chain:[{author:owner,text}],claimed:null,created:Date.now()});localSave();return{};}).then(()=>{$('idea-input').value='';say('Idea accepted. The machine promises not to return it to you.');});
}
function pullIdea(){
  const player=ensureIdentity(); if(!player)return;
  if(held){say('You already have an idea. Twist it or put it back before pulling again.',true);return;}
  act('pull',{player},()=>{
    const candidates=state.ideas.filter(i=>i.owner.toLowerCase()!=player.toLowerCase()&&!i.claimed&&i.chain.at(-1).author.toLowerCase()!=player.toLowerCase());
    if(!candidates.length)throw new Error('No eligible idea is waiting. Invite another thinker or submit ideas under another player.');
    const idea=candidates[Math.floor(Math.random()*candidates.length)]; idea.claimed=player; held=idea.id; localSave(); return{held:idea.id};
  }).then(result=>{held=result?.held||held;say('CLUNK! The machine has issued someone else’s idea. Add an unexpected twist.');render();});
}
function passTwist(){
  const player=ensureIdentity(), text=$('twist-input').value.trim(); if(!player)return;
  if(!held){say('Pull the lever before attempting to twist reality.',true);return;} if(text.length<8){say('The twist needs at least 8 characters.',true);return;}
  act('twist',{player,id:held,text},()=>{const idea=state.ideas.find(i=>i.id===held);if(!idea||idea.claimed!==player)throw new Error('That idea is no longer assigned to this player.');idea.chain.push({author:player,text});idea.claimed=null;localSave();return{};}).then(()=>{held=null;$('twist-input').value='';say('Twist attached. The idea is back inside the machine, noticeably weirder.');render();});
}
function releaseHeld(){
  if(!held)return; const player=name();
  act('release',{player,id:held},()=>{const idea=state.ideas.find(i=>i.id===held);if(idea&&idea.claimed===player)idea.claimed=null;localSave();return{};}).then(()=>{held=null;say('Idea returned to the machine without modification.');render();});
}
function vote(award,id){
  const player=ensureIdentity();if(!player)return;
  act('vote',{player,award,id},()=>{state.votes[award] ||= {};state.votes[award][player]=id;localSave();return{};}).then(()=>say(`Vote recorded for “${award}.”`));
}
function reset(){ if(!confirm('Erase every local idea, twist, and vote on this device?'))return;state=blank();state.ideas=seeds();localSave();held=null;revealOpen=false;say('The local machine has been reset with six starter ideas.');render(); }
function finalConcept(i){return i.chain.map((s,n)=>`${n?'Twist':'Original'}: ${s.text}`).join(' → ')}
function winner(award){const votes=state.votes[award]||{};const counts={};Object.values(votes).forEach(id=>counts[id]=(counts[id]||0)+1);const pair=Object.entries(counts).sort((a,b)=>b[1]-a[1])[0];return pair?`${state.ideas.find(i=>i.id===pair[0])?.chain[0].text||'Unknown'} (${pair[1]})`:'No votes yet';}
function render(){
  const player=name(); $('mode').textContent=api?'PUBLIC MULTIPLAYER':'PASS-THE-PHONE';
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
}
export async function initIdeaSwapMachine(){
  $('category').innerHTML=options();$('player-name').value=sessionStorage.getItem('idea-swap-name')||'';
  if(api){try{const response=await fetch(`${api}/api/state`);if(!response.ok)throw new Error(`HTTP ${response.status}`);const data=await response.json();state=data.state||data;say('Public Multiplayer connected. Ideas can travel between devices.');}catch(e){state=blank();say(`Public Multiplayer could not connect: ${e.message}. Remove ?api= from the URL to use Pass-the-Phone Mode.`,true);}}
  else{state=localLoad();if(!state.ideas.length){state.ideas=seeds();localSave();}say('Pass-the-Phone Mode is ready. Six starter ideas are already rattling inside.');}
  $('save-name').onclick=()=>{if(ensureIdentity()){say(`Player “${name()}” is at the controls.`);render();}};
  $('submit-idea').onclick=submitIdea;$('pull-lever').onclick=pullIdea;$('pass-twist').onclick=passTwist;$('release-held').onclick=releaseHeld;
  $('reveal-button').onclick=()=>{revealOpen=!revealOpen;render();};$('reset-machine').onclick=reset;
  $('idea-input').oninput=e=>$('idea-count').textContent=`${e.target.value.length}/280`;
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
        <section class="panel"><h2>{"Who is pulling the lever?"}</h2><p class="panel-intro">{"Use the same name for your turn. The machine will never hand you your own original idea."}</p><div class="identity"><input id="player-name" maxlength="40" placeholder="Player name or nickname"/><button id="save-name" class="button secondary">{"Take the Controls"}</button></div></section>
        <section class="machine-grid">
          <div class="panel"><h2>{"1. Feed the machine"}</h2><p class="panel-intro">{"Contribute one gloriously strange starting point."}</p><label class="field">{"Category"}<select id="category"></select></label><label class="field">{"Original idea"}<textarea id="idea-input" maxlength="280" placeholder="A coffee mug that warns you before someone schedules a Monday meeting…"></textarea></label><div class="submit-row"><span id="idea-count" class="counter">{"0/280"}</span><button id="submit-idea" class="button primary">{"Deposit Idea 💡"}</button></div></div>
          <div class="panel"><h2>{"2. Tempt the machine"}</h2><p class="panel-intro">{"Pull the lever and accept whatever questionable brilliance appears."}</p><div class="lever-box"><div><div class="lever"></div><button id="pull-lever" class="button primary lever-button">{"PULL THE LEVER 🔄"}</button></div></div></div>
        </section>
        <section id="held" class="held"><span id="held-category" class="category"></span><p id="held-original" class="idea-text"></p><p id="held-chain" class="chain-mini"></p><label class="field">{"3. Add one unexpected twist"}<textarea id="twist-input" maxlength="280" placeholder="It communicates only through dramatic movie-trailer narration…"></textarea></label><div class="twist-actions"><button id="pass-twist" class="button primary">{"Attach Twist + Pass It On"}</button><button id="release-held" class="button secondary">{"Put It Back"}</button></div></section>
        <section class="panel"><h2>{"Machine telemetry"}</h2><p class="panel-intro">{"A responsible dashboard for deeply irresponsible innovation."}</p><div class="stats"><div class="stat"><span>{"IDEAS"}</span><strong id="stat-ideas">{"0"}</strong></div><div class="stat"><span>{"TWISTS"}</span><strong id="stat-twists">{"0"}</strong></div><div class="stat"><span>{"YOUR IDEAS"}</span><strong id="stat-mine">{"0"}</strong></div><div class="stat"><span>{"WAITING"}</span><strong id="stat-waiting">{"0"}</strong></div></div></section>
        <section class="panel"><div class="reveal-head"><div><h2>{"End-of-Day Reveal Vault"}</h2><p class="panel-intro">{"Open every journey, reveal the contributors, and vote for glorious nonsense."}</p></div><button id="reveal-button" class="button secondary">{"Reveal Every Journey + Vote"}</button></div><div id="reveal-body" hidden=true><div id="journeys" class="journeys"></div><h2>{"🏆 Cast your votes"}</h2><div id="votes" class="vote-grid"></div></div><div class="toolbar"><button id="reset-machine" class="button danger">{"Reset Local Machine"}</button></div><p class="mode-note">{"Pass-the-Phone stores everything on this device. Add ?api=https://YOUR-WORKER to the URL when the optional multiplayer service is deployed."}</p></section>
        <p class="slogan">{"Sometimes a wild idea does not need to be practical—it just needs to inspire the next great idea."}</p>
      </main>
    }
}

fn main(){ yew::Renderer::<App>::new().render(); }
