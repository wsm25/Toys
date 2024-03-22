# 上海教育考试院查分小工具
import requests, time, os

url=input("输入网址：")
print("检查高考是否可以查询（五秒刷新一次，可以了之后自动打开）")
while True:
    response = requests.get(url)
    if response.status_code == 200: # 没开前是 301 跳转，导致刷新无效
        break
    print("看起来还不行哦",end='\r')
    time.sleep(5)
os.system("start "+url)
print("可以查啦~")
input()
