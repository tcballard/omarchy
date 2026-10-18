import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,readFile,writeFile,cp,rm,access} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join,resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
const repo=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const helper=join(repo,'tools/agent-team');

test('Overlay installs and restores without replacing the manifest or usage/account code',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'agents-install-'));
  try{
    const source=join(dir,'feature'),target=join(dir,'desktop'),home=join(dir,'home'),bin=join(dir,'bin');
    await mkdir(join(source,'tools'),{recursive:true});await mkdir(join(source,'shell/plugins/agents'),{recursive:true});
    await cp(helper,join(source,'tools/agent-team'),{recursive:true,filter:p=>!p.includes('/target')});
    await cp(join(repo,'shell/plugins/agents'),join(source,'shell/plugins/agents'),{recursive:true});
    await mkdir(join(source,'tools/agent-team/target/release'),{recursive:true});
    await writeFile(join(source,'tools/agent-team/target/release/omarchy-agent-team'),'#!/bin/bash\nprintf "fixture backend\\n"\n',{mode:0o755});
    await mkdir(join(target,'shell/plugins/agents'),{recursive:true});await mkdir(bin);await mkdir(home);
    for(const name of ['Panel.qml','Main.qml','Agent.qml','manifest.json']){
      const original=execFileSync('git',['show',`a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e:shell/plugins/agents/${name}`],{cwd:repo});
      await writeFile(join(target,'shell/plugins/agents',name),original);
    }
    for(const command of ['cargo','systemctl','omarchy-restart-shell'])await writeFile(join(bin,command),'#!/bin/bash\nexit 0\n',{mode:0o755});
    const env={...process.env,HOME:home,OMARCHY_PATH:target,XDG_STATE_HOME:join(home,'state'),XDG_CONFIG_HOME:join(home,'config'),PATH:bin+':'+process.env.PATH};
    const script=join(source,'tools/agent-team/scripts/install');
    execFileSync('bash',[script],{env});
    const unchanged=['Main.qml','Agent.qml','manifest.json'];
    for(const name of unchanged)assert.deepEqual(await readFile(join(target,'shell/plugins/agents',name)),execFileSync('git',['show',`a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e:shell/plugins/agents/${name}`],{cwd:repo}));
    assert.match(await readFile(join(target,'shell/plugins/agents/Panel.qml'),'utf8'),/TasksView/);
    assert.throws(()=>execFileSync('bash',[script],{env,stdio:'pipe'}),/overlay already exists/);
    const panel=join(target,'shell/plugins/agents/Panel.qml');const installed=await readFile(panel,'utf8');
    await writeFile(panel,installed+'\n// later edit\n');
    const restore=join(target,'tools/agent-team/scripts/restore');
    assert.throws(()=>execFileSync('bash',[restore],{env,stdio:'pipe'}),/changed after installation/);
    await writeFile(panel,installed);execFileSync('bash',[restore],{env});
    assert.deepEqual(await readFile(panel),execFileSync('git',['show','a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e:shell/plugins/agents/Panel.qml'],{cwd:repo}));
    await assert.rejects(access(join(target,'tools/agent-team')));
    await writeFile(panel,'new upstream panel');
    assert.throws(()=>execFileSync('bash',[script],{env,stdio:'pipe'}),/changed from the supported upstream/);
    assert.equal(await readFile(panel,'utf8'),'new upstream panel');
  }finally{await rm(dir,{recursive:true,force:true})}
});
