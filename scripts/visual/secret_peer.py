#!/usr/bin/env python3
"""Real Rust C-ABI peer for Secret Chat API tests and native UI recordings.

Runs only in the CI test environment. No private keys are written to fixtures,
logs or video artifacts. The helper decrypts UI messages before acknowledging.
"""
import argparse
import ctypes as C
import json
import os
import sys
import uuid
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from urllib.error import HTTPError
from scenario import post

SUITE = 'TEKTALK_X25519_MLKEM768_AES256GCMSIV_SHA256'

def library():
    name = 'tektalk_client_core.dll' if os.name == 'nt' else ('libtektalk_client_core.dylib' if sys.platform == 'darwin' else 'libtektalk_client_core.so')
    lib = C.CDLL(str(Path('target/release', name).resolve()))
    signatures = {
        'device_create': ([C.c_char_p, C.c_char_p], C.c_void_p),
        'device_destroy': ([C.c_void_p], None),
        'public_bundle_json': ([C.c_void_p], C.c_void_p),
        'begin_epoch_json': ([C.c_void_p, C.c_char_p, C.c_char_p, C.c_uint64, C.c_char_p], C.c_void_p),
        'accept_key_json': ([C.c_void_p, C.c_char_p, C.c_char_p, C.c_char_p, C.c_uint64, C.c_char_p], C.c_int32),
        'encrypt_json': ([C.c_void_p, C.c_char_p, C.c_uint64, C.c_char_p], C.c_void_p),
        'decrypt_json': ([C.c_void_p, C.c_char_p], C.c_void_p),
    }
    for name, (args, result) in signatures.items():
        fn = getattr(lib, 'tektalk_secret_' + name); fn.argtypes = args; fn.restype = result
    lib.tektalk_string_destroy.argtypes = [C.c_void_p]
    lib.tektalk_string_destroy.restype = None
    return lib

def encoded(value):
    return json.dumps(value, separators=(',', ':')).encode()

class Device:
    def __init__(self, lib, tokens):
        self.lib, self.tokens = lib, tokens
        self.handle = lib.tektalk_secret_device_create(tokens['user_id'].encode(), tokens['device_id'].encode())
        assert self.handle, 'Rust device creation failed'
        self.bundle = json.loads(self.take(lib.tektalk_secret_public_bundle_json(self.handle)))
        self.request('/v1/secret/devices/register', {
            'suite': SUITE, 'x25519_public': self.bundle['x25519_public'], 'ml_kem_768_public': self.bundle['ml_kem_768_public']})
    def take(self, pointer):
        assert pointer, 'Rust cryptographic operation failed'
        try: return C.string_at(pointer).decode()
        finally: self.lib.tektalk_string_destroy(pointer)
    def request(self, path, body):
        return post(path, body, self.tokens['access_token'])
    def policy(self, conversation):
        return self.request('/v1/secret/conversations/policy', {'conversation_id': conversation})
    def publish(self, conversation, policy):
        bundles = self.request('/v1/secret/conversations/bundles', {'conversation_id': conversation})
        bundles = [{**b, 'suites': [b['suite']]} for b in bundles]
        packets = json.loads(self.take(self.lib.tektalk_secret_begin_epoch_json(self.handle, conversation.encode(), policy['kind'].encode(), policy['epoch'], encoded(bundles))))
        self.request('/v1/secret/keys/publish', {'conversation_id': conversation, 'epoch': policy['epoch'], 'packets': [{'recipient_device_id': p['recipient_device_id'], 'packet': p} for p in packets]})
    def inbox(self, conversation, policy):
        for packet in self.request('/v1/secret/keys/inbox', {'conversation_id': conversation}):
            assert self.lib.tektalk_secret_accept_key_json(self.handle, conversation.encode(), packet['sender_device_id'].encode(), policy['kind'].encode(), packet['epoch'], encoded(packet['packet'])) == 1
    def decrypt(self, row):
        return self.take(self.lib.tektalk_secret_decrypt_json(self.handle, encoded(row)))
    def send(self, conversation, text, policy):
        wire = json.loads(self.take(self.lib.tektalk_secret_encrypt_json(self.handle, conversation.encode(), policy['epoch'], text.encode())))
        wire.pop('sender_device_id')
        wire['client_message_id'] = str(uuid.uuid4())
        return self.request('/v1/secret/messages/send', wire)['message']

def register(name):
    return post('/v1/auth/register', dict(phone='+84' + str(uuid.uuid4().int % 10**10).zfill(10), display_name=name, password='Learning1234', security_question='City?', security_answer='hanoi', device_name=name))

def verify():
    lib = library()
    alice = Device(lib, register('Secret Alice'))
    bob_tokens = register('Secret Bob')
    bob = Device(lib, bob_tokens)
    carol = Device(lib, register('Secret Carol'))
    # Two actual authenticated devices belonging to the same account.
    phone = '+84' + str(uuid.uuid4().int % 10**10).zfill(10)
    multi = post('/v1/auth/register', dict(phone=phone, display_name='Multi Device', password='Learning1234', security_question='City?', security_answer='hanoi', device_name='phone'))
    login = post('/v1/auth/login', dict(phone=phone, password='Learning1234', device_name='laptop'))
    second = post('/v1/auth/device/verify', dict(challenge_id=login['challenge_id'], answer='hanoi'))
    devices = [alice, bob, carol, Device(lib, multi), Device(lib, second)]
    evidence = []
    for kind, members, recipients in [('one_to_one', [bob_tokens['user_id']], [alice, bob]), ('group', [bob_tokens['user_id'], carol.tokens['user_id'], multi['user_id']], devices)]:
        conversation = str(uuid.uuid4())
        policy = alice.request('/v1/secret/conversations/enable', dict(conversation_id=conversation, kind=kind, member_ids=members))
        for device in recipients: device.publish(conversation, policy)
        for device in recipients: device.inbox(conversation, policy)
        row = alice.send(conversation, 'confidential ' + kind, policy)
        assert 'body' not in row and 'text' not in row
        for device in recipients:
            tampered = {**row, 'ciphertext': [0] * len(row['ciphertext'])}
            assert not lib.tektalk_secret_decrypt_json(device.handle, encoded(tampered)), 'Tampering accepted'
            assert device.decrypt(row) == 'confidential ' + kind
            assert not lib.tektalk_secret_decrypt_json(device.handle, encoded(row)), 'Replay accepted'
        # Repeat opt-in/key discovery must preserve sender key/counter.
        alice.publish(conversation, policy)
        for device in recipients: device.inbox(conversation, policy)
        reply = bob.send(conversation, 'encrypted reply', policy)
        assert alice.decrypt(reply) == 'encrypted reply'
        repeated = alice.send(conversation, 'same epoch still works', policy)
        assert repeated['counter'] == row['counter'] + 1
        assert bob.decrypt(repeated) == 'same epoch still works'
        try:
            alice.request('/v1/chat/messages/send', dict(conversation_id=conversation, recipient_id=bob_tokens['user_id'], client_message_id=str(uuid.uuid4()), text='forbidden plaintext'))
            raise AssertionError('Plaintext downgrade accepted')
        except HTTPError as error: assert error.code == 409
        if kind == 'group':
            new_policy = alice.request('/v1/secret/conversations/enable', dict(conversation_id=conversation, kind=kind, member_ids=[bob_tokens['user_id'], multi['user_id']]))
            assert new_policy['epoch'] == policy['epoch'] + 1
            try:
                carol.request('/v1/secret/messages/history', dict(conversation_id=conversation))
                raise AssertionError('Removed group member retained access')
            except HTTPError as error: assert error.code == 401
            try:
                bob.send(conversation, 'stale epoch', policy)
                raise AssertionError('Stale epoch accepted')
            except HTTPError as error: assert error.code == 409
            for device in [alice, bob, devices[3], devices[4]]: device.publish(conversation, new_policy)
            for device in [alice, bob, devices[3], devices[4]]: device.inbox(conversation, new_policy)
            rotated = alice.send(conversation, 'new membership epoch', new_policy)
            assert bob.decrypt(rotated) == 'new membership epoch'
        evidence.append(dict(kind=kind, recipient_devices=len(recipients), e2ee=True, bidirectional=True, tamper_rejected=True, replay_rejected=True, plaintext_downgrade_rejected=True))
    Path('recordings').mkdir(exist_ok=True)
    Path('recordings/secret-api-proof.json').write_text(json.dumps(evidence, indent=2))
    print('Real Secret Chat API + Rust Core: 1:1, group, multi-device, reply, tamper, replay and downgrade tests passed')

def serve(fixture):
    f = json.loads(fixture.read_text())
    import base64
    claims = json.loads(base64.urlsafe_b64decode(f['peer_token'].split('.')[1] + '==='))
    peer = Device(library(), dict(user_id=f['peer_id'], device_id=claims['device'], access_token=f['peer_token']))
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_): pass
        def do_GET(self):
            self.send_response(200); self.end_headers(); self.wfile.write(b'ready')
        def do_POST(self):
            try:
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                conversation = body['conversation_id']
                policy = peer.policy(conversation)
                peer.inbox(conversation, policy); peer.publish(conversation, policy)
                rows = peer.request('/v1/secret/messages/history', {'conversation_id': conversation})
                outgoing = rows[-1]
                assert 'body' not in outgoing
                assert peer.decrypt(outgoing) == body['outgoing']
                peer.send(conversation, body['incoming'], policy)
                report = dict(platform=f['platform'], kind=policy['kind'], rust_core_decrypted_ui_message=True, encrypted_reply_persisted=True)
                proof = Path('recordings', f['platform'] + '-secret-' + policy['kind'] + '.json')
                proof.write_text(json.dumps(report, indent=2))
                self.send_response(200); self.end_headers(); self.wfile.write(b'{"verified":true}')
            except Exception as error:
                self.send_response(500); self.end_headers(); self.wfile.write(str(error).encode())
    HTTPServer(('127.0.0.1', 8766), Handler).serve_forever()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(); parser.add_argument('command', choices=['verify', 'serve']); parser.add_argument('--fixture', type=Path)
    args = parser.parse_args()
    if args.command == 'verify': verify()
    else: serve(args.fixture)
