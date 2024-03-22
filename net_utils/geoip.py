# 感谢 IP.SB 提供的公共 API！
# !pip install curl_cffi
import requests, sys, json
from curl_cffi import requests

baseurl="https://api.ip.sb/geoip/"
printList={
    "组织名称": ["organization"],
    "地理位置": ["country", "region", "city"]
}

if __name__ == "__main__" and len(sys.argv)>1:
    url=baseurl+sys.argv[1]
    response = requests.get(url, impersonate="chrome110").json()
    # refer to https://pypi.org/project/curl-cffi
    for key in printList:
        res=[]
        for i in printList[key]:
            if i in response:
                res.append(response[i])
        if not res:
            res.append("未知")
        print(key, "\t", " ".join(res))
