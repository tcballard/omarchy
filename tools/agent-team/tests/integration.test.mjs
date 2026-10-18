import {test} from 'node:test';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp, mkdir, writeFile, readFile, chmod, stat, rm, access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createConnection} from 'node:net';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const binary=process.env.TEAM_TEST_BINARY||join(root,'target/debug/omarchy-agent-team');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,ms=15000){const start=Date.now();while(Date.now()-start<ms){const v=await fn();if(v)return v;await pause(50)}throw Error('Timed out waiting for fixture')}
async function harness(){
  const dir=await mkdtemp(join(tmpdir(),'agent-team-test-'));
  const home=join(dir,'home'),runtime=join(dir,'runtime'),state=join(dir,'state'),bin=join(dir,'bin'),cwd=join(dir,'project with spaces');
  for(const p of [home,runtime,state,bin,cwd,join(dir,'account A'),join(dir,'account B')])await mkdir(p,{recursive:true});
  const source=(await readFile(join(root,'tests/fixture-agent.mjs'),'utf8')).replace('#!/usr/bin/env node',`#!${process.execPath}`);
  for(const provider of ['codex','claude']){await writeFile(join(bin,provider),source,{mode:0o755})}
  await writeFile(join(dir,'account-choice'),join(dir,'account A'));
  await writeFile(join(bin,'omarchy-agent-account-home'),`#!/bin/sh\ncat '${dir}/account-choice'\n`,{mode:0o755});
  await writeFile(join(bin,'omarchy-default-agent'),'#!/bin/sh\nprintf codex\n',{mode:0o755});
  const env={...process.env,HOME:home,XDG_STATE_HOME:state,XDG_RUNTIME_DIR:runtime,PATH:bin+':'+process.env.PATH,FIXTURE_TRACE:join(dir,'trace'),FIXTURE_PID:join(dir,'pid'),OPENAI_API_KEY:'fixture-key-should-be-removed',FIXTURE_CHOICE:join(dir,'account-choice'),FIXTURE_NEXT_ACCOUNT:join(dir,'account B')};
  let daemon;
  const launch=async()=>{
    await rm(join(runtime,'omarchy-agent-team/endpoint.json'),{force:true});
    daemon=spawn(binary,['daemon'],{env,stdio:['ignore','pipe','pipe']});let error='';daemon.stderr.on('data',b=>error+=b);daemon.stdout.on('data',b=>error+=b);
    await until(async()=>{if(daemon.exitCode!==null)throw Error(error);try{await access(join(runtime,'omarchy-agent-team/endpoint.json'));return true}catch{return false}});
  };
  await launch();
  const rpc=async(op,params={},extra={})=>{
    const p=spawn(binary,['request'],{env,stdio:['pipe','pipe','pipe']});let out='';p.stdout.on('data',b=>out+=b);p.stdin.end(JSON.stringify({op,params,...extra}));
    await new Promise(r=>p.on('exit',r));return JSON.parse(out);
  };
  const task=async(id)=>{const r=await rpc('read',{taskId:id});assert.equal(r.ok,true,r.error);return r.data};
  const done=id=>until(async()=>{const t=await task(id);return !['queued','running'].includes(t.status)&&t});
  return {dir,home,state,runtime,cwd,env,rpc,task,done,launch,get daemon(){return daemon},async close(){daemon.kill('SIGTERM');await new Promise(r=>daemon.exitCode!==null||daemon.signalCode!==null?r():daemon.on('exit',r));await rm(dir,{recursive:true,force:true})}};
}

test('Codex lead delegates through real MCP bridge to Claude and deduplicates retries',async()=>{
  const h=await harness();try{
    const started=await h.rpc('start',{provider:'default',cwd:h.cwd,task:'fixture:delegate'});assert.equal(started.ok,true,started.error);
    const result=await h.done(started.data.id);assert.equal(result.status,'completed',result.error);assert.match(result.result,/Lead collected independent review/);
    const list=(await h.rpc('list')).data.tasks;assert.equal(list.length,2);assert.equal(list.filter(t=>t.parent===result.id).length,1);
    const trace=(await readFile(join(h.dir,'trace'),'utf8')).trim().split('\n').map(JSON.parse);
    assert.equal(trace.length,2);assert.equal(trace.every(t=>t.cwd===h.cwd),true);assert.equal(trace.every(t=>!t.apiKeyPresent),true);
    assert.equal(trace.find(t=>!t.claude).sandbox,'read-only');assert.equal(trace.find(t=>t.claude).tools,'Read,Glob,Grep');
    assert.equal((await stat(join(h.runtime,'omarchy-agent-team/endpoint.json'))).mode&0o777,0o600);
    assert.equal((await stat(join(h.state,'omarchy/agent-team/tasks.json'))).mode&0o777,0o600);
    const persisted=await readFile(join(h.state,'omarchy/agent-team/tasks.json'),'utf8');assert.equal(persisted.includes('capability'),false);
  }finally{await h.close()}
});

test('Claude lead delegates to Codex; account changes only affect new task launches',async()=>{
  const h=await harness();try{
    const first=await h.rpc('start',{provider:'claude',cwd:h.cwd,task:'fixture:delegate'});const result=await h.done(first.data.id);assert.equal(result.status,'completed',result.error);
    await writeFile(join(h.dir,'account-choice'),join(h.dir,'account B'));
    const second=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:delegate'});assert.equal((await h.done(second.data.id)).status,'completed');
    const trace=(await readFile(join(h.dir,'trace'),'utf8')).trim().split('\n').map(JSON.parse);
    assert.equal(trace.length,4);assert.equal(trace.slice(0,2).every(t=>t.home.endsWith('account A')),true);assert.equal(trace.slice(2).every(t=>t.home.endsWith('account B')),true);
  }finally{await h.close()}
});

test('Cancellation propagates to child and stops the owned worker process',async()=>{
  const h=await harness();try{
    const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:slowchild'});assert.equal(start.ok,true);
    await until(async()=>{try{await access(join(h.dir,'pid'));return true}catch{return false}});
    const pid=Number(await readFile(join(h.dir,'pid'),'utf8'));
    assert.equal((await h.rpc('cancel',{taskId:start.data.id})).ok,true);
    await until(async()=>{try{process.kill(pid,0);return false}catch{return true}});
    const list=(await h.rpc('list')).data.tasks;assert.equal(list.length,2);assert.equal(list.every(t=>t.status==='cancelled'),true);
  }finally{await h.close()}
});

test('Provider errors, output overflow and invalid requests remain visible',async()=>{
  const h=await harness();try{
    for(const [brief,pattern] of [['fixture:fail',/Authentication/],['fixture:overflow',/output exceeded/]]){
      const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:brief});const result=await h.done(start.data.id);assert.equal(result.status,'failed');assert.match(result.error,pattern);
    }
    assert.equal((await h.rpc('start',{provider:'grok',cwd:h.cwd,task:'x'})).ok,false);
    assert.equal((await h.rpc('start',{provider:'codex',cwd:'relative',task:'x'})).ok,false);
    assert.equal((await h.rpc('start',{provider:'codex',cwd:h.home,task:'x'})).ok,false);
    assert.equal((await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'x'.repeat(12001)})).ok,false);
    assert.equal((await h.rpc('list',{}, {parent:'forged',capability:'forged'})).ok,false);
  }finally{await h.close()}
});

test('Restart preserves results and marks interrupted work without replaying it',async()=>{
  const h=await harness();try{
    const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:sleep'});await until(async()=>(await h.task(start.data.id)).status==='running');
    await pause(200);h.daemon.kill('SIGTERM');await new Promise(r=>h.daemon.on('exit',r));
    await h.launch();await until(async()=>(await h.task(start.data.id)).status==='interrupted');
    assert.match((await h.task(start.data.id)).error,/interrupted/);
    assert.equal((await h.rpc('list')).data.tasks.length,1);
  }finally{await h.close()}
});

test('Active task ceiling and local transport authentication are enforced',async()=>{
  const h=await harness();try{
    const ids=[];
    for(let i=0;i<4;i++){
      const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:sleep'});assert.equal(start.ok,true,start.error);ids.push(start.data.id);
    }
    const fifth=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:sleep'});assert.equal(fifth.ok,false);assert.match(fifth.error,/Four tasks/);
    const endpoint=JSON.parse(await readFile(join(h.runtime,'omarchy-agent-team/endpoint.json'),'utf8'));
    const response=await new Promise((resolve,reject)=>{
      const options=endpoint.kind==='tcp'?{host:'127.0.0.1',port:Number(endpoint.address.split(':').at(-1))}:{path:join(h.runtime,'omarchy-agent-team/broker.sock')};
      const socket=createConnection(options);let output='';socket.on('error',reject);socket.on('data',b=>output+=b);socket.on('end',()=>resolve(JSON.parse(output)));socket.on('connect',()=>socket.end(JSON.stringify({token:'wrong',request:{op:'list'}})+'\n'));
    });
    assert.equal(response.ok,false);assert.match(response.error,/transport credential/);
    for(const id of ids)assert.equal((await h.rpc('cancel',{taskId:id})).ok,true);
  }finally{await h.close()}
});

test('A corrupt durable history is preserved and startup fails visibly',async()=>{
  const h=await harness();try{
    h.daemon.kill('SIGTERM');await new Promise(r=>h.daemon.on('exit',r));
    const path=join(h.state,'omarchy/agent-team/tasks.json');await writeFile(path,'broken-history');
    const p=spawn(binary,['daemon'],{env:h.env,stdio:['ignore','pipe','pipe']});let output='';p.stdout.on('data',b=>output+=b);const code=await new Promise(r=>p.on('exit',r));
    assert.equal(code,1);assert.match(output,/Cannot load history/);assert.equal(await readFile(path,'utf8'),'broken-history');
  }finally{await h.close()}
});


test('Both accounts are pinned at submission even if selection changes before delegation',async()=>{
  const h=await harness();try{
    const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:account-change'});
    assert.equal((await h.done(start.data.id)).status,'completed');
    const trace=(await readFile(join(h.dir,'trace'),'utf8')).trim().split('\n').map(JSON.parse);
    assert.equal(trace.length,2);
    assert.equal(trace.every(t=>t.home.endsWith('account A')),true);
  }finally{await h.close()}
});

test('A lead that skips delegation cannot report a completed two-agent workflow',async()=>{
  const h=await harness();try{
    const start=await h.rpc('start',{provider:'codex',cwd:h.cwd,task:'fixture:no-delegate'});
    const result=await h.done(start.data.id);
    assert.equal(result.status,'failed');assert.match(result.error,/without a completed independent review/);
  }finally{await h.close()}
});

test('Top-level retry keys cannot return work from a different project',async()=>{
  const h=await harness();try{
    const params={provider:'codex',cwd:h.cwd,task:'fixture:delegate',clientRequestId:'same'};
    const start=await h.rpc('start',params);assert.equal(start.ok,true);
    const other=join(h.dir,'other');await mkdir(other);
    const retry=await h.rpc('start',{...params,cwd:other});assert.equal(retry.ok,false);assert.match(retry.error,/different request/);
    await h.done(start.data.id);
  }finally{await h.close()}
});


test('Discovery names all four subscriptions without advertising unimplemented adapters',async()=>{
  const h=await harness();try{
    const doctor=await h.rpc('doctor');assert.equal(doctor.ok,true);
    assert.deepEqual(doctor.data.providers.map(p=>p.provider),['codex','claude','grok','kimi']);
    for(const p of doctor.data.providers){
      assert.match(p.authentication,/not verified by discovery/);
      if(['grok','kimi'].includes(p.provider)){
        assert.equal(p.supported,false);assert.deepEqual(p.roles,[]);
        assert.match(p.executionStatus,/adapter pending/);
        const task=await h.rpc('start',{provider:p.provider,cwd:h.cwd,task:'review'});
        assert.equal(task.ok,false);assert.match(task.error,/no execution adapter/);
      }
    }
  }finally{await h.close()}
});
