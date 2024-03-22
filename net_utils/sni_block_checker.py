# 可能需要更新以下的测试 ip，暂时依赖于 curl 的版本和平台（因为依赖于输出格式）
from subprocess import PIPE, Popen

# curl  -oNUL https://${domain}  --resolve "*:443:${ip}"
def test(ip:str,name:str,website:str):
	if(Debug):
		print("\n"+"="*20)
	print('测试 '+name+' IP：\t',end='')
	cmd='curl  --connect-timeout 5 -oNUL https://' + website + ' --resolve "*:443:' + ip + '"'
	p = Popen(cmd, shell=True, stderr=PIPE)
	if(Debug):
		print(cmd)
	text = str(p.communicate()[1])
	# if(Debug):
	#  	print(text)
	if(p.returncode == 0):
		print("连接成功")
	elif("SNI or certificate check failed" in text): 
		print("连接成功，但是域名与 ip 不符")
	elif("failed to receive handshake" in text):
		print("SNI RESET 阻断")
	elif("Connection refused" in text): # only BandwagonHost and DMIT
		print("拒绝连接")
	elif("SSL/TLS connection timeout" in text):
		print("SNI 超时阻断")
	elif("next InitializeSecurityContext failed" in text):
		print("TLS握手失败（可能是加密套件不匹配）") # Z-Library uses it to defend against domain detection
	elif("SEC_E_UNTRUSTED_ROOT" in text):
		print("不受信任的根证书")
	elif("Connection timeout" in text):
		print("超时")
	elif("Empty reply from server" in text):
		print("连接成功，但下载中断")
	else:
		print("未知错误，Output:\n",text,'\n\n'+'='*20)

America=[
  ["1.1.1.1", "Cloudflare"],
  ["104.244.42.65", "Twitter"],
]

Europe=[
    ["91.198.174.192", "荷兰"],
]
Asia=[
  ["47.75.19.144", "阿里云香港"], # oss-cn-hongkong.aliyuncs.com
]

Block=[
  ["199.59.148.147", "推特新加坡"],
  ["66.220.149.32", "Facebook 俄勒冈"],
]

AllList=Asia+Europe+America
Simplelist=[America[0],Asia[1]]

SwitchedList=Simplelist
Debug=False

if __name__=='__main__': 
	domain=input("请输入测试域名：")
	while domain=="":
		domain=input("请输入测试域名：")
	ip=input("请输入测试 IP（可选）：")
	if not ip=="":
		test(ip,"自选",domain)
	else:
		for l in SwitchedList:
			test(l[0],l[1],domain)
	input("测试结束！输入回车退出")
