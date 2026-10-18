#!/usr/bin/env node
import {spawn} from 'node:child_process';
import {readFileSync, writeFileSync, appendFileSync} from 'node:fs';
import {createInterface} from 'node:readline';

const args=process.argv.slice(2);
if(args.includes('--version')){console.log('Fixture CLI 1.0');process.exit(0)}
if(args.includes('--help')){console.log('--restricted --output-format --mcp-config');process.exit(0)}
let prompt='';for await(const chunk of process.stdin)prompt+=chunk;
const claude=args.includes('-p');
let bridge;
if(claude){bridge=JSON.parse(args[args.indexOf('--mcp-config')+1]).mcpServers.agent_team}
else {
  const config=args.find(a=>a.startsWith('mcp_servers={agent_team='));
  if(config){
    const quoted=key=>JSON.parse(config.match(new RegExp(key+'=("(?:[^"\\\\]|\\\\.)*")'))[1]);
    bridge={command:quoted('command'),args:['mcp'],env:{AGENT_TEAM_TASK:quoted('AGENT_TEAM_TASK'),AGENT_TEAM_CAPABILITY:quoted('AGENT_TEAM_CAPABILITY')}};
  }
}
if(process.env.FIXTURE_TRACE)appendFileSync(process.env.FIXTURE_TRACE,JSON.stringify({lead:!!bridge,claude,cwd:process.cwd(),home:(claude?process.env.CLAUDE_CONFIG_DIR:process.env.CODEX_HOME)||null,apiKeyPresent:!!process.env.OPENAI_API_KEY,tools:claude?args[args.indexOf('--tools')+1]:null,sandbox:claude?null:args[args.indexOf('--sandbox')+1]})+'\n');
const emit=text=>console.log(JSON.stringify(claude?{type:'result',result:text,is_error:false}:{type:'item.completed',item:{type:'agent_message',text}}));
if(prompt.includes('fixture:overflow')){process.stdout.write('x'.repeat(70_000));await new Promise(()=>{})}
if(prompt.includes('fixture:fail')){console.log(JSON.stringify({type:'result',result:'Authentication required (fixture)',is_error:true}));process.exit(1)}
if(prompt.includes('fixture:sleep')){if(process.env.FIXTURE_PID)writeFileSync(process.env.FIXTURE_PID,String(process.pid));setInterval(()=>{},60000);await new Promise(()=>{})}
if(prompt.includes('fixture:no-delegate')){emit('Lead skipped the independent review.');process.exit(0)}
if(bridge){
  const proc=spawn(bridge.command,bridge.args,{env:{...process.env,...bridge.env},stdio:['pipe','pipe','inherit']});
  const lines=createInterface({input:proc.stdout});let seq=0;const pending=new Map();
  lines.on('line',line=>{let v=JSON.parse(line);pending.get(v.id)?.(v);pending.delete(v.id)});
  const call=(method,params={})=>new Promise(resolve=>{const id=++seq;pending.set(id,resolve);proc.stdin.write(JSON.stringify({jsonrpc:'2.0',id,method,params})+'\n')});
  const tool=async(name,arguments_)=>{const v=await call('tools/call',{name,arguments:arguments_});if(v.result.isError)throw Error(v.result.content[0].text);return JSON.parse(v.result.content[0].text)};
  try{
    const init=await call('initialize',{protocolVersion:'2024-11-05',capabilities:{},clientInfo:{name:'fixture',version:'1'}});
    if(!init.result.capabilities.tools)throw Error('MCP initialization failed');
    const list=await call('tools/list');if(list.result.tools.length!==4)throw Error('Missing tools');
    const provider=claude?'codex':'claude';
    const task=prompt.includes('fixture:slowchild')?'fixture:sleep':'Review the error path (fixture)';
    const brief={provider,task,clientRequestId:'review-1'};
    if(prompt.includes('fixture:account-change'))writeFileSync(process.env.FIXTURE_CHOICE,process.env.FIXTURE_NEXT_ACCOUNT);
    const same=await call('tools/call',{name:'delegate_task',arguments:{...brief,provider:claude?'claude':'codex'}});
    if(!same.result.isError)throw Error('Same-provider delegation accepted');
    const child=await tool('delegate_task',brief);
    const retry=await tool('delegate_task',brief);if(retry.id!==child.id)throw Error('Duplicate retry created work');
    const changed=await call('tools/call',{name:'delegate_task',arguments:{...brief,task:'different brief'}});
    if(!changed.result.isError)throw Error('Retry key accepted a changed brief');
    const unrelated=await call('tools/call',{name:'task_status',arguments:{taskId:bridge.env.AGENT_TEAM_TASK}});
    if(!unrelated.result.isError)throw Error('Scope escaped');
    const forged=await call('tools/call',{name:'delegate_task',arguments:{...brief,cwd:'/tmp'}});
    if(!forged.result.isError)throw Error('Unexpected cwd accepted');
    let result;
    do {result=await tool('task_status',{taskId:child.id,waitMs:1000})}while(['queued','running'].includes(result.status));
    if(result.status!=='completed')throw Error(result.error||result.status);
    emit('Lead collected independent review: '+result.result);
  }finally{proc.stdin.end();lines.close()}
}else emit('Independent review complete: src/example.rs:4 has a fixture finding.');
