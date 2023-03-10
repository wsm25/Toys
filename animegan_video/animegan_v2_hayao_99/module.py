import os

from paddlehub import Module
from paddlehub.module.module import moduleinfo

from animegan_v2_hayao_99.model import Model
from animegan_v2_hayao_99.processor import Processor

@moduleinfo(
    name="animegan_v2_hayao_99", # 模型名称
    type="CV/style_transfer", # 模型类型
    author="jm12138", # 作者名称
    author_email="jm12138@qq.com", # 作者邮箱
    summary="animegan_v2_hayao_99", # 模型介绍
    version="1.0.2" # 版本号
)
class Animegan_V2_Hayao_99(Module):
    # 初始化函数
    def __init__(self, use_gpu=False):
        # 设置模型路径
        self.directory="animegan_v2_hayao_99"
        self.model_path = os.path.join(self.directory, "animegan_v2_hayao_99")

        # 加载模型
        self.model = Model(self.model_path, use_gpu)     
    
    # 关键点检测函数
    def style_transfer(
        self,
        images=None,
        output_dir='output',
        visualization=False,
        min_size=32,
        max_size=1024
    ):
        # 加载数据处理器
        processor = Processor(
            images, 
            output_dir, 
            min_size, 
            max_size
        )

        # 模型预测
        outputs = self.model.predict(processor.input_datas)

        # 结果后处理
        results = processor.postprocess(outputs, visualization)

        # 返回结果
        return results
