#!/bin/sh

/bin/busybox --install -s /bin

echo "Starting test glibc..."

mkdir -p /lib
cp -rv /ext4/glibc/lib/* /lib

mkdir -p /test
cp -rv /ext4/glibc/busybox /test/
# for item in /ext4/glibc/*; do
#     [ "$item" = "/ext4/glibc/lib" ] && continue
#     echo "Copying $item to /test..."
#     cp -a "$item" /test/
# done

# # 3. 要执行的脚本列表（数组形式）
# SCRIPTS="basic_testcode.sh busybox_testcode.sh"

# # 遍历并执行每个脚本
# for script in $SCRIPTS; do
#     if [ -e /test/$script ]; then
#         echo ">> Running $script..."
#         sh /test/$script
#     else
#         echo "/test/$script does not exist or is not executable."
#     fi
# done

cd /test

LIB_NAME=glibc

# ==================================================

# echo ">> Running basic_testcode.sh..."
# cp -rv /ext4/$LIB_NAME/basic /test/
# chmod +x /test/basic/run-all.sh
# cp -rv /ext4/$LIB_NAME/basic_testcode.sh /test/
# sh /test/basic_testcode.sh
# rm -rf /test/basic
# rm -rf /test/basic_testcode.sh
# echo ">> Finished basic_testcode.sh"

# ==================================================

# echo ">> Running busybox_testcode.sh..."
# cp -rv /ext4/$LIB_NAME/busybox_cmd.txt /test/
# cp -rv /ext4/$LIB_NAME/busybox_testcode.sh /test/
# sh /test/busybox_testcode.sh
# rm -rf /test/busybox_cmd.txt
# rm -rf /test/busybox_testcode.sh
# echo ">> Finished busybox_testcode.sh"

# ==================================================

# 实现系统调用 196

# echo ">> Running iozone_testcode.sh..."
# cp -rv /ext4/$LIB_NAME/iozone /test/
# cp -rv /ext4/$LIB_NAME/iozone_testcode.sh /test/
# chmod +x /test/iozone
# sh /test/iozone_testcode.sh
# rm -rf /test/iozone
# rm -rf /test/iozone_testcode.sh
# echo ">> Finished iozone_testcode.sh"

# ==================================================

echo ">> Running libctest_testcode.sh..."
cp -rv /ext4/$LIB_NAME/run-static.sh /test/
cp -rv /ext4/$LIB_NAME/run-dynamic.sh /test/
cp -rv /ext4/$LIB_NAME/libctest_testcode.sh /test/
cp -rv /ext4/$LIB_NAME/runtest.exe /test/
cp -rv /ext4/$LIB_NAME/entry-dynamic.exe /test/
cp -rv /ext4/$LIB_NAME/entry-static.exe /test/
chmod +x /test/run-static.sh
chmod +x /test/run-dynamic.sh
chmod +x /test/runtest.exe
chmod +x /test/entry-dynamic.exe
chmod +x /test/entry-static.exe
sh /test/libctest_testcode.sh
rm -rf /test/run-static.sh
rm -rf /test/run-dynamic.sh
rm -rf /test/libctest_testcode.sh
rm -rf /test/runtest.exe
rm -rf /test/entry-dynamic.exe
rm -rf /test/entry-static.exe
echo ">> Finished libctest_testcode.sh"

# ==================================================

# echo ">> Running libcbench_testcode.sh..."
# cp -rv /ext4/$LIB_NAME/libc-bench /test/
# cp -rv /ext4/$LIB_NAME/libcbench_testcode.sh /test/
# chmod +x /test/libc-bench
# sh /test/libcbench_testcode.sh
# rm -rf /test/libc-bench
# rm -rf /test/libcbench_testcode.sh
# echo ">> Finished libcbench_testcode.sh"

# ==================================================

# echo ">> Running lua_testcode.sh..."
# cp -rv /ext4/$LIB_NAME/date.lua     /test/
# cp -rv /ext4/$LIB_NAME/file_io.lua  /test/
# cp -rv /ext4/$LIB_NAME/max_min.lua  /test/
# cp -rv /ext4/$LIB_NAME/random.lua   /test/
# cp -rv /ext4/$LIB_NAME/remove.lua   /test/
# cp -rv /ext4/$LIB_NAME/round_num.lua /test/
# cp -rv /ext4/$LIB_NAME/sin30.lua    /test/
# cp -rv /ext4/$LIB_NAME/sort.lua     /test/
# cp -rv /ext4/$LIB_NAME/strings.lua  /test/
# cp -rv /ext4/$LIB_NAME/test.sh      /test/
# cp -rv /ext4/$LIB_NAME/lua          /test/
# cp -rv /ext4/$LIB_NAME/lua_testcode.sh /test/
# chmod +x /test/test.sh
# chmod +x /test/lua
# sh /test/lua_testcode.sh
# rm -rf /ext4/$LIB_NAME/date.lua     
# rm -rf /ext4/$LIB_NAME/file_io.lua  
# rm -rf /ext4/$LIB_NAME/max_min.lua  
# rm -rf /ext4/$LIB_NAME/random.lua   
# rm -rf /ext4/$LIB_NAME/remove.lua   
# rm -rf /ext4/$LIB_NAME/round_num.lua
# rm -rf /ext4/$LIB_NAME/sin30.lua    
# rm -rf /ext4/$LIB_NAME/sort.lua     
# rm -rf /ext4/$LIB_NAME/strings.lua  
# rm -rf /ext4/$LIB_NAME/test.sh      
# rm -rf /ext4/$LIB_NAME/lua          
# rm -rf /ext4/$LIB_NAME/lua_testcode.sh
# echo ">> Finished lua_testcode.sh"



# rm -rf /lib/*
# rm -rf /test/*