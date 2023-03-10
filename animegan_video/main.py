'''
use animegan of paddle to stylize a video 
author: wsm
date:   2022.05.06
'''
# import
import av
from tqdm import tqdm


# params here
inputVideo='input.mp4'
outputVidio='output.mp4'
outBufferSize=10*1024#10*1024*1024 # 10 MB
usegpu=False
visualization=False # set to output every frame (maybe to make a preview of output)

# input video
fi=av.open(inputVideo)
  # frame size & batch size
frm1=fi.decode(video=0).__next__()
hh,ww=frm1.height,frm1.width
  # frame rate
stm1=fi.streams.video[0]
fr=stm1.guessed_rate
  # frame number
frn=stm1.frames
if frn==0:
    frn=None
fi.close()
fi=av.open(inputVideo)


# process frames while output
  # init output
outFile=av.open(outputVidio, mode="w",buffer_size=outBufferSize)
outStream = outFile.add_stream("libx264", rate=fr)
outStream.width , outStream.height=ww//32*32,hh//32*32 # animegan may cut the frame
outStream.pix_fmt = "yuv420p"
  # init paddle
from animegan_v2_hayao_99.module import Animegan_V2_Hayao_99
model = Animegan_V2_Hayao_99(use_gpu=usegpu)
  # process
for frm in tqdm(fi.decode(video=0),total=frn):
    imgTmp=frm.to_ndarray(format="bgr24")
    # imgTmp=[imgTmp]
    # stylize
    imgTmp = model.style_transfer(images=imgTmp,visualization=visualization,max_size=max(ww,hh))# [0]
    # output
    imgTmp = av.VideoFrame.from_ndarray(imgTmp, format="bgr24")
    for packet in outStream.encode(imgTmp):
        outFile.mux(packet)
    # break


# something to clear
  # Flush stream
for packet in outStream.encode():
    outFile.mux(packet)
  # Close the file
outFile.close()
