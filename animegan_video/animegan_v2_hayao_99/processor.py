import os
import cv2 
import time
import numpy as np

__all__ = ['Processor']

def check_dir(dir_path):
    # 目录检查函数
    if not os.path.exists(dir_path):
        os.makedirs(dir_path)
    elif os.path.isfile(dir_path):
        os.remove(dir_path)
        os.makedirs(dir_path)

class Processor():
    # 初始化函数
    def __init__(
        self, 
        images=None, 
        output_dir='output',
        min_size=32,
        max_size=1024
    ):
        # 变量设置
        self.min_size = min_size
        self.max_size = max_size

        self.images = images
        self.output_dir = output_dir

        # 获取原始输入数据
        self.datas = self.images

        # 对原始输入数据进行预处理
        self.input_datas = self.preprocess()


    # 数据预处理函数
    def preprocess(self):
        img=self.datas
            # 格式转换
        img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        
        # 缩放图片
        #h, w = img.shape[:2]
        #if max(h,w)>self.max_size:
        #    img = cv2.resize(img, (self.max_size, int(h/w*self.max_size))) if h<w else cv2.resize(img, (int(w/h*self.max_size), self.max_size))
        #elif min(h,w)<self.min_size:
        #    img = cv2.resize(img, (self.min_size, int(h/w*self.min_size))) if h>w else cv2.resize(img, (int(w/h*self.min_size), self.min_size))

        # 裁剪图片
        h, w = img.shape[:2]
        img = img[:h-(h%32), :w-(w%32), :]

        # 归一化
        img = img/127.5 - 1.0

        # 新建维度 so that 我们可以batch（如果显存够用）
        img = np.expand_dims(img, axis=0).astype('float32')

        # 加入输入数据列表
        # input_datas.append(img)

        # 数据按batch_size切分
        # input_datas = np.concatenate(input_datas, 0)
        # split_num = len(self.datas)//self.batch_size+1 if len(self.datas)%self.batch_size!=0 else len(self.datas)//self.batch_size
        # input_datas = np.array_split(input_datas, split_num)   

        # 返回预处理完成的数据
        return img

    def postprocess(self, outputs, visualization):
        # results = []

        # for im_id, output in enumerate(outputs):
            # 反归一化
        image = (outputs.squeeze() + 1.) / 2 * 255

        # 限幅
        image = np.clip(image, 0, 255).astype(np.uint8)

        # 格式转换
        image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)

        # 可视化
        if visualization:
            # 检查输出目录
            check_dir(self.output_dir)

            # 写入输出图片
            cv2.imwrite(os.path.join(self.output_dir, '%d.jpg' % (time.time())), image)

            # results.append(image)

        # 返回结果
        return image