#!/usr/bin/env python3
import ctypes,json,subprocess,time
from pathlib import Path
from pywinauto import Application
from pywinauto.keyboard import send_keys
from scenario import verify_reply
f=json.loads(Path('build/visual-fixtures/windows.json').read_text())
exe=next(p for p in Path('clients/windows').rglob('TEKtalk.exe') if 'Release' in str(p))
app=Application(backend='uia').start(str(exe.resolve()))
window=app.window(title_re='.*TEKtalk.*');window.wait('visible',timeout=40);window.maximize();window.set_focus()
def control(id):
    c=window.child_window(auto_id=id);c.wait('visible enabled',timeout=30);return c.wrapper_object()
def tap(id):control(id).click_input();time.sleep(.9)
def fill(id,text):
    e=control(id);e.click_input();send_keys('^a{BACKSPACE}');send_keys(text.replace('+','{+}').replace('(','{(}').replace(')','{)}'),with_spaces=True,pause=.06);time.sleep(.5)
def visible(text):
    # A sidebar preview can contain the same text; prove it is in the chat pane.
    window.child_window(auto_id='chat.messages').child_window(title=text,control_type='Text').wait('visible',timeout=30)
Path('recordings').mkdir(exist_ok=True)
record=subprocess.Popen(['ffmpeg','-y','-f','gdigrab','-framerate','15','-draw_mouse','1','-i','desktop','-c:v','libx264','-preset','ultrafast','-crf','24','-pix_fmt','yuv420p','recordings/windows.mp4'],stdin=subprocess.PIPE,stdout=open('recordings/windows-recorder.log','wb'),stderr=subprocess.STDOUT)
complete=False
try:
    time.sleep(2);tap('auth.toggle')
    for id,value in [('auth.phone',f['phone']),('auth.name','MVP Learner'),('auth.password',f['password']),('auth.question',f['question']),('auth.answer',f['answer'])]:fill(id,value)
    time.sleep(1.5);tap('auth.submit')
    tabs=control('navigation.bottom').rectangle()
    assert tabs.top>window.rectangle().top+window.rectangle().height()/2, 'Desktop tabs must be at the bottom'
    tap('tab.AI');visible('TEK AI');tap('tab.Me');tap('auth.logout')
    fill('auth.phone',f['phone']);fill('auth.password',f['password']);tap('auth.submit');control('auth.answer');time.sleep(2)
    fill('auth.answer',f['answer']);tap('auth.submit');control('chat.conversation')
    fill('chat.conversation',f['conversation']);fill('chat.recipient',f['peer_id']);fill('chat.draft',f['outgoing']);time.sleep(1.5);tap('chat.send');visible(f['outgoing']);time.sleep(2)
    verify_reply(f);tap('chat.sync');visible(f['incoming'])
    fill('chat.draft','Draft preserved across tabs');tap('tab.AI');tap('tab.Me');tap('tab.Message')
    assert control('chat.draft').get_value()=='Draft preserved across tabs'
    visible(f['incoming']);control('history.'+f['conversation']);time.sleep(3);complete=True
finally:
    if record.poll() is None:
        record.communicate(b'q\n',timeout=30)
    if complete:Path('recordings/windows-flow.json').write_text(json.dumps({'platform':'windows','registration':True,'logout':True,'login':True,'verification':True,'sent_and_received_in_ui':True}))
    else:window.print_control_identifiers(filename='recordings/windows-ui.txt')
