#!/usr/bin/env python3
"""Drive the installed Compose UI through ADB accessibility and screenrecord."""
import json,re,subprocess,time,uuid,xml.etree.ElementTree as ET
from urllib.request import Request,urlopen
from pathlib import Path
from scenario import verify_reply
f=json.loads(Path('build/visual-fixtures/android.json').read_text())
def adb(*args,**kw):return subprocess.run(['adb',*args],check=True,**kw)
def nodes():
    adb('shell','uiautomator','dump','/sdcard/tektalk-ui.xml',stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    data=subprocess.check_output(['adb','exec-out','cat','/sdcard/tektalk-ui.xml'])
    return ET.fromstring(data)
def find(id=None,text=None,timeout=30):
    end=time.time()+timeout
    while time.time()<end:
        for n in nodes().iter('node'):
            if (id and n.get('resource-id')==id) or (text and n.get('text')==text):return n
        time.sleep(.4)
    raise AssertionError(f'Visible UI element not found: {id or text}')
def find_any(*ids,timeout=30):
    end=time.time()+timeout
    while time.time()<end:
        visible={n.get('resource-id') for n in nodes().iter('node')}
        for id in ids:
            if id in visible:return id
        time.sleep(.4)
    raise AssertionError(f'None of the visible UI elements were found: {ids}')
def wait_text(id,expected,timeout=30):
    end=time.time()+timeout
    while time.time()<end:
        node=find(id=id,timeout=1)
        if expected in (node.get('text') or ''):return node
        time.sleep(.4)
    raise AssertionError(f'Visible UI element {id} did not contain: {expected}')
def tap(id):
    n=find(id=id);x1,y1,x2,y2=map(int,re.findall(r'\d+',n.get('bounds')))
    adb('shell','input','tap',str((x1+x2)//2),str((y1+y2)//2));time.sleep(.7)
def fill(id,text):
    tap(id);adb('shell','input','keycombination','113','29');adb('shell','input','keyevent','67')
    adb('shell','input','text',text.replace(' ','%s'));time.sleep(.5)
    adb('shell','input','keyevent','4');time.sleep(.3)
def pause():time.sleep(1.5)
Path('recordings').mkdir(exist_ok=True)
adb('shell','am','force-stop','vn.tektalk')
adb('shell','am','start','-W','-n','vn.tektalk/.presentation.MainActivity',stdout=subprocess.DEVNULL)
find(id='auth.toggle')
record=subprocess.Popen(['adb','shell','screenrecord','--bit-rate','4000000','--time-limit','180','/sdcard/tektalk-mvp.mp4'])
complete=False
try:
    pause();tap('auth.toggle')
    for id,value in [('auth.phone',f['phone']),('auth.name','MVP Learner'),('auth.password',f['password']),('auth.question',f['question']),('auth.answer',f['answer'])]:fill(id,value)
    pause();tap('auth.submit');find(id='tab.2');pause()
    tap('tab.2');tap('auth.logout');find(id='auth.phone');pause()
    fill('auth.phone',f['phone']);fill('auth.password',f['password']);pause();tap('auth.submit')
    destination=find_any('auth.answer','chat.open')
    if destination=='auth.answer':pause();fill('auth.answer',f['answer']);tap('auth.submit')
    find(id='chat.open');pause()
    tap('chat.open');fill('chat.conversation',f['conversation']);fill('chat.recipient',f['peer_id']);fill('chat.draft',f['outgoing']);pause();tap('chat.send');find(text=f['outgoing']);pause()
    verify_reply(f);tap('chat.sync');find(text=f['incoming']);pause();
    if record.poll() is not None:raise RuntimeError('Screen recording ended before the flow completed')
    complete=True
finally:
    subprocess.run(['adb','shell','pkill','-INT','screenrecord'],check=False)
    try:record.wait(timeout=15)
    except subprocess.TimeoutExpired:record.terminate()
    adb('pull','/sdcard/tektalk-mvp.mp4','recordings/android.mp4')
    if complete:Path('recordings/android-flow.json').write_text(json.dumps({'platform':'android','registration':True,'logout':True,'login':True,'verification':True,'sent_and_received_in_ui':True}))

# Record Secret Chat as a second continuous segment because Android's native
# screenrecord has a 180-second limit. Packaging joins both actual recordings.
record=subprocess.Popen(['adb','shell','screenrecord','--bit-rate','4000000','--time-limit','180','/sdcard/tektalk-secret.mp4'])
try:
    tap('chat.secret')
    for kind in ('one_to_one','group'):
        conversation=str(uuid.uuid4())
        fill('chat.conversation',conversation)
        if kind=='group':
            tap('chat.secret.group');fill('chat.secret.members',f['peer_id'])
        tap('chat.secret.enable');wait_text('chat.status','Secret Chat enabled')
        outgoing='Encrypted '+kind+' from Android'
        incoming='Verified encrypted '+kind+' reply'
        fill('chat.draft',outgoing);tap('chat.send');find(text=outgoing)
        data=json.dumps(dict(conversation_id=conversation,outgoing=outgoing,incoming=incoming)).encode()
        with urlopen(Request('http://127.0.0.1:8766/reply',data=data,headers={'Content-Type':'application/json'}),timeout=30) as response:
            assert json.load(response)['verified']
        tap('chat.sync');find(text=incoming);pause()
        tap('chat.sync');find(text=incoming);pause()
    if record.poll() is not None:raise RuntimeError('Secret Chat recording ended before flow completion')
    Path('recordings/android-secret-flow.json').write_text(json.dumps(dict(platform='android',secret_one_to_one=True,secret_group=True,secret_history_reload=True)))
finally:
    subprocess.run(['adb','shell','pkill','-INT','screenrecord'],check=False)
    try:record.wait(timeout=15)
    except subprocess.TimeoutExpired:record.terminate()
    adb('pull','/sdcard/tektalk-secret.mp4','recordings/android-secret.mp4')
