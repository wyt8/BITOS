#!/bin/sh

# 设置工作目录
CWD="/ext4x0/glibc"

for test in basic_testcode.sh busybox_testcode.sh cyclictest_testcode.sh \
            iozone_testcode.sh iperf_testcode.sh libcbench_testcode.sh \
            libctest_testcode.sh lmbench_testcode.sh ltp_testcode.sh \
            lua_testcode.sh netperf_testcode.sh unixbench_testcode.sh; do
    echo "run test $test in $CWD"
    (
        cd "$CWD" || { echo "Cannot enter directory $CWD"; exit 1; }
        /bin/busybox sh "$test"
    )
done

# 测试脚本数组
# TESTS=(
#     "basic_testcode.sh"
#     "busybox_testcode.sh"
#     "cyclictest_testcode.sh"
#     "iozone_testcode.sh"
#     "iperf_testcode.sh"
#     "libcbench_testcode.sh"
#     "libctest_testcode.sh"
#     "lmbench_testcode.sh"
#     "ltp_testcode.sh"
#     "lua_testcode.sh"
#     "netperf_testcode.sh"
#     "unixbench_testcode.sh"
# )

# # 执行测试脚本
# for test in "${TESTS[@]}"; do
#     echo "run test $test in $CWD"
#     (
#         cd "$CWD" || { echo "Cannot enter directory $CWD"; exit 1; }
#          sh "$test"
#     )
# done