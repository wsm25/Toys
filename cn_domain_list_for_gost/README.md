# China domain list for Gost

适用于 Gost 的中国网站屏蔽列表

使用 [Domain list community](https://github.com/v2fly/domain-list-community) 和 [chnroutes2](https://github.com/misakaio/chnroutes2)

```shell
mkdir /etc/gost && cd /etc/gost
# chown gostuser: .
wget https://raw.githubusercontent.com/wsm25/cn-domain-list-gost/main/cndm.txt
wget https://raw.githubusercontent.com/wsm25/cn-domain-list-gost/main/cnip.txt
wget https://raw.githubusercontent.com/wsm25/cn-domain-list-gost/main/gost.yml
gost
```

或者直接 iptables 封掉
```shell
ipset create cn hash:net # maxelem 1000000
iptables -P OUTPUT ACCEPT
iptables -I OUTPUT -m set --match-set cn -p tcp -j DROP
iptables -I OUTPUT --sport 443 -j ACCEPT
iptables -I OUTPUT --sport 80 -j ACCEPT
iptables -I OUTPUT --sport 22 -j ACCEPT
service iptables save

ipset flush cn
curl  https://raw.githubusercontent.com/wsm25/cn-domain-list-gost/main/cnip.txt | while read line; do ipset add cn $line; done
```
