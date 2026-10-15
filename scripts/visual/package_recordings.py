#!/usr/bin/env python3
"""Normalize actual screen recordings to H.264 and combine complete flows."""
import argparse,json,subprocess
from pathlib import Path
ORDER=('web','android','ios','macos','windows')
LABEL={'web':'Web','android':'Android','ios':'iOS','macos':'macOS','windows':'Windows'}
def run(*args):subprocess.run(args,check=True)
def probe(path):
    return json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-show_format','-of','json',str(path)]))
def normalize(platform,source,dest):
    info=probe(source);duration=float(info['format']['duration'])
    if duration<8:raise RuntimeError(f'{platform}: recording too short ({duration}s)')
    run('ffmpeg','-y','-i',str(source),'-vf',f'scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:800:(ow-iw)/2:70+(720-ih)/2:color=0x0b1020,setsar=1,drawtext=fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf:text=TEKtalk - {LABEL[platform]} - real CI recording:x=24:y=22:fontsize=25:fontcolor=white','-an','-r','24','-c:v','libx264','-preset','fast','-crf','23','-pix_fmt','yuv420p','-movflags','+faststart',str(dest))
def main():
    p=argparse.ArgumentParser();p.add_argument('directory',type=Path);a=p.parse_args();out=a.directory/'published';out.mkdir(exist_ok=True)
    for platform in ORDER:
        source=a.directory/(platform+'.mp4')
        if not source.exists():source=a.directory/(platform+'.webm')
        if not source.exists():raise RuntimeError('Missing actual recording for '+platform)
        if not (a.directory/(platform+'-flow.json')).exists() and not (a.directory/(platform+'-backend.json')).exists():raise RuntimeError('Missing completed flow evidence for '+platform)
        normalize(platform,source,out/(platform+'.mp4'))
    listing=out/'concat.txt';listing.write_text('\n'.join("file '"+p+".mp4'" for p in ORDER)+'\n')
    run('ffmpeg','-y','-f','concat','-safe','0','-i',str(listing),'-c','copy','-movflags','+faststart',str(out/'TEKtalk_MVP_All_Platforms.mp4'))
    report={p:{'duration_seconds':float(probe(out/(p+'.mp4'))['format']['duration']),'format':'H.264 MP4','source':'continuous real screen recording'} for p in ORDER}
    (out/'recording-manifest.json').write_text(json.dumps(report,indent=2))
if __name__=='__main__':main()
