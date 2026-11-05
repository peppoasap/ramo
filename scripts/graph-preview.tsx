// Development-only gesture harness. No native bridge or Git writes.
import React, {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {History} from '../src/components/History';
import '../src/styles.css';
const commits = [0,1,2].map(i=>({oid: String(i).repeat(40), parents:i<2?[String(i+1).repeat(40)]:[], author:'Synthetic',date:'2026-11-04',subject:['Search feature','Main work','Initial project'][i], refs:''}));
const branches = [{name:'feature',oid:commits[0].oid,remote:false},{name:'main',oid:commits[1].oid,remote:false}];
function Harness() {
 const [result,setResult] = useState('No drop');
 return <main style={{height:'100vh',display:'flex',flexDirection:'column'}}><output role="status" style={{padding:20}}>{result}</output><History branches={branches} currentBranch="main" commits={commits} selected={null} onSelect={()=>setResult('Selected')} filter="" onFilter={()=>{}} onMore={()=>{}} hasMore={false} busy={false} onBranchContext={()=>setResult('Branch menu')} onCommitContext={()=>setResult('Commit menu')} dragContext="synthetic:fixed" onGraphDrop={(source,target,context)=>setResult(`${source.kind} → ${target?.subject ?? 'current main'} · ${context}`)} /></main>;
}
createRoot(document.getElementById('root')!).render(<Harness/>);
