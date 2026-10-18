import {test} from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {readFile,mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join,resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
const repo=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const widget=join(repo,'shell/plugins/agents');

async function launch(home,omarchy){
  const panel=await readFile(join(widget,'Panel.qml'),'utf8');
  const source=await readFile(join(widget,'TeamPrompt.js'),'utf8');
  const events=[];const context=vm.createContext({JSON,Quickshell:{env:k=>({HOME:home,OMARCHY_PATH:omarchy})[k]},root:{close:()=>events.push('closed')},Util:{execArgv:args=>events.push(args)}});
  vm.runInContext(source,context);context.TeamPrompt={build:context.build};
  for(const name of ['startPrompt','startTeam']){
    const body=panel.match(new RegExp('  function '+name+'\\([^)]*\\) \\{([\\s\\S]*?)\\n  \\}'))?.[1];
    assert.ok(body,`${name} missing`);
    vm.runInContext(`function ${name}(${name==='startPrompt'?'prompt':''}) {${body}\n}`,context);
  }
  context.startTeam();return {panel,events};
}

test('Pointer and keyboard orchestration close the panel and use the Themes/Plugins native launcher',async()=>{
  const {panel,events}=await launch('/home/test','/opt/omarchy');
  assert.equal(events[0],'closed');assert.equal(events.length,2);
  assert.equal(events[1][0],'omarchy-agent-prompt');assert.equal(events[1].length,2);
  assert.match(panel,/onClicked: root\.startTeam\(\)/);
  assert.match(panel,/target\.kind === "tasks"\) startTeam\(\)/);
  assert.doesNotMatch(panel,/TasksView|tasksLoader|tasksView|showTasks/);
});

test('The terminal handoff bounds execution and preserves paths as prompt data',async()=>{
  const home='/home/name with "quotes";$(false)';const root='/opt/custom path';
  const {events}=await launch(home,root);const prompt=events[1][1];
  assert.ok(prompt.includes(JSON.stringify(home+'/.local/bin/omarchy-agent-team')));
  assert.ok(prompt.includes(JSON.stringify(root+'/tools/agent-team/scripts/setup-backend')));
  assert.match(prompt,/Do not start a review until these are clear/);
  assert.match(prompt,/only reviewers supported by doctor/);
  assert.match(prompt,/Kimi is not enabled/);
  assert.match(prompt,/never interpolate my path or brief/);
  assert.match(prompt,/taskId:ID,waitMs:10000/);
});

test('The real agent-prompt/agent/TUI chain delegates to the configured default terminal',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'agent-team-terminal-'));
  try{
    const bin=join(dir,'bin');await mkdir(bin);const trace=join(dir,'terminal.json');
    const scripts={
      'omarchy-default-agent':'printf codex',
      'omarchy-cmd-missing':'exit 1',
      'omarchy-agent-account-home':'exit 0',
      'setsid':'exec "$@"',
      'uwsm-app':'[[ $1 == -- ]] && shift; exec "$@"',
    };
    for(const [name,body] of Object.entries(scripts))await writeFile(join(bin,name),'#!/bin/bash\n'+body+'\n',{mode:0o755});
    await writeFile(join(bin,'xdg-terminal-exec'),`#!${process.execPath}\nrequire('fs').writeFileSync(process.env.TERMINAL_TRACE,JSON.stringify(process.argv.slice(2)))\n`,{mode:0o755});
    const brief='review "quoted" path; $(touch should-not-exist)';
    execFileSync(join(repo,'bin/omarchy-agent-prompt'),[brief],{cwd:dir,env:{...process.env,HOME:dir,OMARCHY_PATH:repo,PATH:bin+':'+join(repo,'bin')+':'+process.env.PATH,TERMINAL_TRACE:trace}});
    const args=JSON.parse(await readFile(trace,'utf8'));
    assert.equal(args[0],'--app-id=org.omarchy.agent');assert.equal(args[1],'-e');
    assert.equal(args.includes('codex'),true);assert.equal(args.at(-1),brief);
    assert.equal(args.includes('--approve-for-me'),true);
    assert.equal(args.includes('bash'),false);
  }finally{await rm(dir,{recursive:true,force:true})}
});
