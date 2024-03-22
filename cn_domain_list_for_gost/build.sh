# abandoned for terrible efficiency
data_path='../domains/data/'
cndm_file='./cndm.txt'

# referring to https://github.com/v2fly/domain-list-community
handle_file(){ # param: file
    if [ -z $1 ] || [ ! "-a $1" ] ; then return 0; fi; # check file
    cat ${data_path}/$1 | while read line
    do
        line=$(echo $line | sed 's/#.*$//') # clear comment
        if [ -n $line ] ; then continue; fi; # skip empty lines
        
        # include
        inc=$(echo $line | grep "include:")
        if [ -n "${inc//include:/}" ] ; then
            handle_file ${inc//include:/}
            continue;
        fi
        # other
        if [ -n "$(echo $line | grep "keyword:")" ] ; then continue; fi;
        # look into issue https://github.com/v2fly/domain-list-community/issues/390
        echo $line | sed  's/^domain:\|^regexp:\|^full://' | sed 's/@.*$//'>> $cndm_file
        
    done
}
