import os

base_address = 0x80400000
step = 0x20000
linker = 'src/linker.ld'

app_id = 0
apps = os.listdir('src/bin') #获取目录下的文件
apps.sort() #排序

for app in apps:
    app = app[:app.find('.')] #获取文件名
    lines = []
    lines_before = []
    with open(linker, 'r') as f:
        for line in f.readlines():
            lines_before.append(line) #保存初始文件
            line = line.replace(hex(base_address),hex(base_address+step*app_id)) #替换BASE_ADDRESS地址
            lines.append(line) #保存替换后的文件
    with open(linker, 'w+') as f:
        f.writelines(lines) #更改后的文件写进linker.ld
    os.system('cargo build --bin %s --release' % app) #编译 --bin %s 是指定你想要构建的二进制目标的名称，只构建某一个应用。
    print('[build.py] application %s start with address %s' %(app, hex(base_address+step*app_id)))
    with open(linker, 'w+') as f:
        f.writelines(lines_before) #恢复成初始文件
    app_id = app_id + 1
