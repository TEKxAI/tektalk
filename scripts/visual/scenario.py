#!/usr/bin/env python3
"""Real local API/PostgreSQL fixtures. No mock endpoints or fabricated messages."""
import argparse,json,os,secrets,subprocess,time,uuid
from pathlib import Path
from urllib.request import Request,urlopen

API='http://127.0.0.1:8080'
def post(path,body,token=None):
    headers={'Content-Type':'application/json'}
    if token:headers['Authorization']='Bearer '+token
    with urlopen(Request(API+path,data=json.dumps(body).encode(),headers=headers),timeout=30) as r:return json.load(r)
def backend():
    env={**os.environ,'JWT_SECRET':secrets.token_hex(32),'OTP_HMAC_SECRET':secrets.token_hex(32),'APP_BIND':'0.0.0.0:8080','RUST_LOG':'info'}
    Path('recordings').mkdir(exist_ok=True)
    log=open('recordings/backend.log','wb')
    flags={'creationflags':subprocess.CREATE_NEW_PROCESS_GROUP} if os.name=='nt' else {'start_new_session':True}
    proc=subprocess.Popen(['target/release/chat-server'+('.exe' if os.name=='nt' else '')],env=env,stdout=log,stderr=log,**flags)
    for _ in range(90):
        if proc.poll() is not None:raise RuntimeError('Backend exited; inspect recordings/backend.log')
        try:
            with urlopen(API+'/readyz',timeout=1) as r:
                if r.status==200:return
        except Exception:time.sleep(1)
    raise RuntimeError('Backend did not become ready')
def prepare(platform,path):
    suffix=str(int(time.time()*1000))[-8:]
    f={'platform':platform,'phone':'+849'+suffix,'password':'Learning1234','answer':'hanoi','question':'Favourite city?','conversation':str(uuid.uuid4()),'outgoing':'Hello from '+platform+' MVP','incoming':'Reply received from the real backend'}
    peer=post('/v1/auth/register',dict(phone='+848'+suffix,display_name='MVP Peer',password=f['password'],security_question=f['question'],security_answer=f['answer'],device_name='recording-peer'))
    f['peer_id']=peer['user_id'];f['peer_token']=peer['access_token']
    path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(f));return f

def verify_reply(f):
    for _ in range(30):
        rows=post('/v1/chat/messages/history',{'conversation_id':f['conversation'],'limit':50},f['peer_token'])
        matches=[r for r in rows if r['body']==f['outgoing']]
        if matches:break
        time.sleep(1)
    else:raise RuntimeError('Outgoing message was not persisted/visible to recipient')
    sent=matches[0]
    reply=post('/v1/chat/messages/send',{'conversation_id':f['conversation'],'recipient_id':sent['sender_id'],'client_message_id':str(uuid.uuid4()),'text':f['incoming']},f['peer_token'])['message']
    rows=post('/v1/chat/messages/history',{'conversation_id':f['conversation'],'limit':50},f['peer_token'])
    assert {f['outgoing'],f['incoming']}.issubset({r['body'] for r in rows})
    report={'platform':f['platform'],'backend':'real Rust API + PostgreSQL','outgoing_persisted':True,'reply_persisted':True,'message_ids':[str(sent['id']),str(reply['id'])]}
    Path('recordings').mkdir(exist_ok=True);Path('recordings/'+f['platform']+'-backend.json').write_text(json.dumps(report,indent=2));return report
def verify_existing(f):
    rows=post('/v1/chat/messages/history',{'conversation_id':f['conversation'],'limit':50},f['peer_token'])
    assert {f['outgoing'],f['incoming']}.issubset({r['body'] for r in rows})
    report={'platform':f['platform'],'backend':'real Rust API + PostgreSQL','outgoing_persisted':True,'reply_persisted':True}
    Path('recordings/'+f['platform']+'-backend.json').write_text(json.dumps(report,indent=2))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('command',choices=['backend','prepare','verify-reply','verify-existing']);p.add_argument('--platform');p.add_argument('--fixture',type=Path);a=p.parse_args()
    if a.command=='backend':backend()
    elif a.command=='prepare':prepare(a.platform,a.fixture)
    elif a.command=='verify-reply':verify_reply(json.loads(a.fixture.read_text()))
    else:verify_existing(json.loads(a.fixture.read_text()))
