#!/usr/bin/env python3
"""Synthetic Ollama-shaped transport, no model/inference. Refuses other model names.

Only bind if port 11434 is free. --state-dir holds request/disconnect counters.
Create `respond` there to return a fixed synthetic message instead of waiting.
This is a manual QA server; never package or use it as a provider adapter.
"""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse
import hashlib
import json
import select
import threading
import time
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--state-dir',type=Path,required=True)
args=parser.parse_args()
args.state_dir.mkdir(parents=True,exist_ok=True)
lock=threading.Lock()
def event(kind, **values):
    with lock:
        with (args.state_dir/'events.jsonl').open('a') as stream:
            stream.write(json.dumps({'event':kind, **values})+'\n')
class Handler(BaseHTTPRequestHandler):
    protocol_version='HTTP/1.1'
    def log_message(self,*args): pass
    def reply(self,value,status=200):
        body=json.dumps(value).encode()
        self.send_response(status);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.send_header('Connection','close');self.end_headers()
        try:self.wfile.write(body)
        except (BrokenPipeError,ConnectionResetError):pass
    def do_GET(self):
        if self.path=='/api/tags':self.reply({'models':[{'name':'ramo-cancel-fixture'}]})
        else:self.reply({'error':'Synthetic fixture route only'},404)
    def do_POST(self):
        size=int(self.headers.get('Content-Length','0'))
        if self.path!='/api/chat' or not 0<size<=150_000:self.reply({'error':'Synthetic fixture only'},400);return
        body=self.rfile.read(size)
        value=json.loads(body)
        if value.get('model')!='ramo-cancel-fixture':self.reply({'error':'Only the synthetic model identifier is allowed'},400);return
        event('request',bytes=len(body),sha256=hashlib.sha256(body).hexdigest())
        if (args.state_dir/'respond').exists():
            self.reply({'message':{'content':'test: synthetic cancellation fixture'}});event('synthetic-response');return
        self.connection.setblocking(False)
        deadline=time.monotonic()+60
        while time.monotonic()<deadline:
            readable,_,_=select.select([self.connection],[],[],0.1)
            if readable:
                try:closed=not self.connection.recv(1024)
                except ConnectionResetError:closed=True
                if closed:event('disconnected');return
        event('fixture-timeout')
        self.close_connection=True
server=ThreadingHTTPServer(('127.0.0.1',11434),Handler)
event('ready',model='ramo-cancel-fixture')
print('Synthetic transport listening at 127.0.0.1:11434; no model or inference.',flush=True)
server.serve_forever()
